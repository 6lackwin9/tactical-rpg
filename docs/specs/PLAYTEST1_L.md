# PLAYTEST1-L: playtest recordings, notes, in-game playback and headless replay

Status: approved (user 2026-10-17, all open-question defaults accepted; design-brainstorm 2026-10-17)
Depends on: none (uses the merged `CombatSession`, `CombatPlan`, `EncounterSource`, Scene Runner)
Lane: logic only (no asset part). Commit 1 (core, tool, suite) first; commit 2 (UI) depends on commit 1.
Touches: new `scripts/playtest/*`, new `scripts/ui/playtest/*`, `scripts/combat/session/combat_session.gd` (signals only), `scenes/combat_arena.gd` (playback, seed, take-over fields), `scripts/ui/runner/scene_runner.gd` (hub child, Playtests panel), new `tools/replay.gd`, `tools/fc.sh` (`replay` command), new `scripts/tests/playtest_test_suite.gd` + one line in `scripts/tests/suite_registry.gd`, new `playtests/README.md`; STRUCTURE rows and README delta via the planner

## Goal

Tushar plays on Windows (`run.bat` Scene Runner, debug build); every battle he plays is recorded as a small versioned JSON file (encounter, seed, content fingerprint, confirmed plans per round, outcome per round) plus a UI event trail and his F8 feedback notes with screenshots. A cloud agent replays the file headlessly (`tools/replay.gd`), checks it reproduces, explains divergence or staleness, renders the noted moments under Xvfb and tries seeds or a changed round on its side. Tushar can open, scrub, replay and take over any recording in the game UI. Human-facing material stays small: his own recordings, at most one agent repro per question, and scenarios that are one Scene Runner entry with one chosen seed (user 2026-10-17). Debug builds only (`OS.is_debug_build()`); release builds record nothing.

## Schema contract (logic lane; schema first: yes, commit 1)

All JSON is written with `JSON.stringify(data, "  ", true)` through `JsonUtil` (or the same helper) to `<path>.tmp`, then renamed (atomic). Ints that may exceed 2^53 (RNG state) are decimal strings. Coordinates are `[x, y]`. Unknown keys are kept on load and ignored; missing keys take the defaults below.

### A. Recording `user://playtests/<id>.json`

| Field | Type, allowed values | Default | Validation |
| --- | --- | --- | --- |
| `format` | `"playtest"` | - | required, else refused |
| `version` | int, `1` | - | > 1 refused ("newer recording"), < 1 refused |
| `id` | `YYYYMMDD-HHMMSS` local time, `-2`, `-3` on collision | - | matches file name |
| `origin` | `user`, `agent` | `user` | enum |
| `created`, `engine`, `source_rev` | ISO time; `Engine.get_version_info().string`; git rev read best effort from `res://.git` | `""` | free text, empty allowed |
| `scenario` | scenario id (H) or `""` | `""` | - |
| `parent` | `{id, battle, round}` for a take-over branch, or `null` | `null` | `round` >= 1 |
| `content` | `{algo: "sha256-12", total, files: {res path: hash}}` | - | required for replay; see rule 3 |
| `battles` | Array of B | `[]` | - |
| `trail` | Array of F; `trail_dropped` int counter beside it | `[]`, `0` | max 20000 events |
| `notes` | Array of G | `[]` | `n` unique, 1-based |

### B. Battle

| Field | Value | Default |
| --- | --- | --- |
| `index` | 0-based position in `battles` | - |
| `encounter` | `EncounterSource` id (`goblin_ambush`, `ashlands:skirmish`) | required; `EncounterSource.exists` else the battle is refused |
| `seed` | int actually used (`encounter.rng_seed` at `start()`) | required |
| `seed_source` | `encounter`, `scenario`, `branch`, `tool` | `encounter` |
| `roster` | `[{ref, catalog_id, name}]` for allies then enemies, session order | required |
| `rounds` | Array of C, ascending, no gaps | `[]` |
| `end` | `{result: won, lost, fled, timeout, abandoned, rounds: int}`; `won/lost/fled` from `BattleBridge.outcome`, `timeout` = max rounds reached, `abandoned` = scene left first | `abandoned` |

### C. Round

| Field | Value |
| --- | --- |
| `round` | int >= 1 |
| `rng_state` | string: `session.resolver.rng.state` read when `round_started` fires |
| `plans` | `[{unit: ref, steps: [D]}]` in `plan.sequences` insertion order (planning order breaks tile-claim ties, PLAN1) |
| `attempts` | int >= 1 (redo count + 1) |
| `discarded` | plans of earlier attempts of this round (same shape as `plans`), oldest first; UX data only, never replayed |
| `plan_ms` | wall ms from `round_started` to the accepted confirm, minus time with the note box open |
| `edits` | count of `plan_edited` events in the accepted attempt |
| `outcome` | E, captured when `round_concluded` fires |

### D. Step codec (`PlanCodec`)

`{mode, skill, target, dir, tiles, cell}`: `mode` = `PlanStep.Mode` key name (`ENGAGE`, `MOVE`, `GOTO`, `AREA`, `WITHDRAW`, `SELF`, `ALLY`); `skill` = `SkillDef.id` or `""`; `target` = unit ref or `null`; `dir` = `direction`; `tiles` int; `cell` (always written, `[-1, -1]` when unset). Unit ref = `{side: ally | enemy, i: index in session.allies / session.enemies, name}`. Decode errors (each names round, unit and step index): unknown mode, ref index out of range, `catalog_id` mismatch with `roster`, skill id not owned by the actor (same lookup the planner's options use). A `name` mismatch alone is a staleness warning. The codec encodes every script property of `PlanStep`; a new property without a codec entry fails the suite (check 1), so S12T/S12C step fields cannot be silently dropped.

### E. Outcome

`{units: [{ref, hp, ap, pos, stance, statuses, withdrawn}], over: bool, log_count: int}`, units in roster order; `statuses` = sorted `["<id>:<rounds left>"]` of active statuses; `stance` = stance name. Compared field by field, exact equality.

### F. Trail event

`{t: ms since recording start, type, ...}`. Controls are described as `ctl = {path: last 4 node-path segments under the current scene, cls, text: first 40 chars of text, tip: first 60 chars of tooltip_text}`.

| `type` | Extra fields | Source |
| --- | --- | --- |
| `scene` | `path` | Scene Runner launch / back to menu |
| `panel`, `modal` | `ctl`, `open: bool` | `visibility_changed` of `CombatPanel` subclasses / `Window` (popups, dialogs) found via `SceneTree.node_added` |
| `button` | `ctl` | `BaseButton.pressed` (connected via `node_added`) |
| `key` | `key` (`OS.get_keycode_string`), `mods` | key presses, skipped while a `LineEdit` / `TextEdit` has focus |
| `hover` | `ctl`, `ms` | `Viewport.gui_get_hovered_control()` polled every 100 ms; logged on leave when dwell >= 700 ms |
| `plan_edit` | `kind` (`add`, `remove`, `move`, `undo`, `clear`, `preset`, `preset_all`), `unit` ref, `round` | `plan_edited` signal |
| `confirm` | `ok: bool`, `reason` (session `status_text` when refused), `round` | `plan_confirmed` / refused confirm |
| `round_reset`, `battle_reset`, `battle_end` | `battle`, `round` | session signals |
| `view` | `round`, `block` | round / block navigation outside the live round (playback-control usage) |
| `note` | `n` | note saved |
| `click` | `button` (left, right, middle, wheel skipped), `pos` `[x, y]`, `ctl` (control under the mouse, interactive or not; `null` over the 3D/2D world), `cell` (`[x, y]` when over the combat grid, else absent), `handled: bool` (a GUI control consumed it) | every mouse button press in `_input` (user 2026-10-17: clicks and hovers in every scene, not only battles) |
| `error` | `level` (`error`, `warning`, `script`), `msg` (first 300 chars), `where` (`file:line` of the top frame when known) | engine and script errors and warnings via a custom `Logger` registered with `OS.add_logger` while recording; max 500 per recording |

### G. Note

`{n, t, origin: user | agent, scene, battle (-1 outside a battle), round, block, phase (planning, resolving, replay or ""), mouse: [x, y], viewport: [w, h], ctl, tags, text, image}`. `tags` subset of `mechanic`, `ui`, `ux`, `bug` (default `[]`); `text` 1..2000 chars; `image` = file name in `user://playtests/<id>/` (`note_<n>.jpg`) or `""` (headless or capture failed). Agent notes have `image: ""`.

### H. Scenario `playtests/scenarios/<name>.json` (agent-written, repo)

`{format: "playtest_scenario", version: 1, id, title, why, encounter, seed}`. One file = one Scene Runner entry = one battle with that seed; `seed` required (the agent picks a representative or interesting one from its sweep). No other configuration in v1.

### I. Variant patch (agent side only, `tools/replay.gd --variant=`)

`{format: "playtest_variant", version: 1, battle, round, plans: [same as C.plans]}`.

### Session additions (`combat_session.gd`, signals only, no behaviour change)

`round_started(round_num)` at the end of `_begin_round`; `plan_confirmed(round_num)` after a successful `confirm_plan`; `round_concluded(round_num, over)` in `_conclude_round`; `round_reset(round_num)` in `reset_round`; `plan_edited(kind, actor)` from the `plan_*` mutators. `mounted` marks `start()` (a new battle entry). `export_log` is unchanged.

## Rules (logic lane)

1. Lifecycle: the `PlaytestHub` node (child of the Scene Runner, survives scene changes) opens one recording per app run on the first launch while the Scene Runner pref `record_playtests` is on (default on). `PlaytestRecorder` (RefCounted, testable) attaches to each new `CombatSession` the arena creates; every `mounted` opens a new battle entry (a battle reset keeps the old entry with `end.result = abandoned`). The file is flushed at each `round_concluded`, note save, scene change and `NOTIFICATION_WM_CLOSE_REQUEST`.
2. Redo: `reset_round` moves the attempt's plans to `discarded`, increments `attempts` and re-reads `rng_state` on the new `round_started`. Live redo behaviour (the RNG keeps advancing) is unchanged; the recording stays exact because replay writes `rng_state` per round.
3. Fingerprint (`ContentFingerprint`): every `*.json` under `res://data/` recursively, sorted path, `FileAccess.get_sha256` cut to 12 hex chars; `total` = the same cut of the SHA-256 of the joined `path:hash\n` lines. Root is a parameter (tests use a sandbox). Stale = totals differ; the report lists added, removed and changed paths. Staleness warns, it never blocks replay.
4. Replay (`PlaytestReplayer`, RefCounted): `EncounterSource.create(encounter)`, set `encounter.rng_seed = seed`, `session.start()`; per round: check `current_round`, set `resolver.rng.state` from `rng_state`, decode plans and `plan.set_steps` in recorded order, require `plan_is_valid`, `confirm_plan(false)`, resolve to REPLAY (as `TestHelpers.resolve_until_replay`), compare the outcome (E), `advance_round` unless the recorded round was the last. Result per round: `match`, `diverge` (first differing unit and field, expected and actual), `plan_broken` (decode or validity error). The first non-match ends verification; later rounds report `unverified`.
5. Take-over and branch: take-over from round N = fresh replay of rounds 1..N-1 (rule 4, never snapshot restore), then the live session plans round N. The hub starts a new recording with `parent = {id, battle, round: N}`, `seed_source = branch` and rounds 1..N-1 copied verbatim. Advancing past the last recorded round in playback is a take-over at that round.
6. In-game playback (commit 2): Scene Runner panel "Playtests" lists recordings from `user://playtests/` and `res://playtests/{inbox,repro}/` (badge `repo`) newest first, with battles, notes, stale badge; buttons Open, Notes, Take over (round picker), Open folder (`OS.shell_open(ProjectSettings.globalize_path("user://playtests"))`), Copy to repo; the absolute folder path is shown as selectable text. Open launches the arena with `playtest`, `battle` set before it enters the tree: the arena fast-forwards with rule 4 before `host.setup(session)` (no FX flood) and then shows the battle with the existing round navigation and playback panel. A banner shows `PLAYBACK <id> battle k round r/n`, the stale badge, a Notes drop-down (jump = `select_round(round)` + `select_block(block)`, block 0 = round start) and "Take over this round". Scenarios (H) are listed in the same panel and launch the encounter with the scenario seed, recorded with `seed_source = scenario`.
7. Notes (commit 2): F8 (handled in `_input`, works in any scene and over popups) first captures `get_viewport().get_texture().get_image()` (before the box appears), then opens a small box: multiline text, four tag toggles, Save (Ctrl+Enter) and Cancel (Esc). Auto-play is paused while it is open. Save writes `note_<n>.jpg` (`save_jpg`, quality 0.85) and the note stamp (G) from the session the hub is attached to and the hovered control at F8 time.
8. Sharing: Copy to repo copies the recording and its image folder to `res://playtests/inbox/<id>/` (writable when run from source); the user pushes it on any branch and tells the agent the id. Agent repro files go to `playtests/repro/`, scenarios to `playtests/scenarios/`; `playtests/README.md` states this layout and the one-scenario rule.
9. Headless tool `tools/replay.gd` (`godot --headless --path . -s tools/replay.gd -- <file> [args]`, wrapped as `tools/fc.sh replay`): lines start with `§` like the other tools; `--battle=K` (default all); `--json` (machine-readable report). Exit 0 all match (stale only warns), 1 divergence or broken plan, 2 load or validation error.
10. Variant: `--variant=<patch>` replays rounds before the patch round unchanged, applies its plans, then `--then=stop` (default) or `--then=recorded` (later recorded plans, `plan_broken` stops); prints per round `DIFF <ref> <field> <recorded> -> <variant>` and the two `end` results. `--write=<out>` saves the variant run as a recording (`origin: agent`), the only way an agent produces a repro for the user.
11. Seed sweep: `--seeds=<list or a..b>` (max 200 per call) re-runs the recorded plans under each seed with rule 4 minus outcome comparison (seed replaces `seed`, `rng_state` writes are skipped). When a plan breaks: `--fallback=stop` (default; result `broken@r`) or `--fallback=<PlanPresets id>` (`plan_apply_preset_all` from that round on). Summary: counts of won, lost, fled, timeout, broken; median and min..max of rounds and party HP left in percent; the recorded seed's row; outliers = seeds whose rounds or HP left fall outside 1.5 IQR, or whose result differs from the majority. It is a debugging aid on one recording, not a balance tool (balance stays `tools/fc.sh sim`, balance lane); large sweeps run only when the planner allows CPU.
12. Trail report (agent side, user 2026-10-17: find interactions that feel unintuitive or break expected behaviour even when the screen looks fine): `--trail-report` reads the trail of every scene, not only battles, and prints `FLAG <kind> t=<ms> <ctl> <detail>` lines for: `dead_click` (a `click` with `handled = false` or on a non-interactive control, followed within 500 ms by no `button`, `panel`, `modal`, `plan_edit`, `confirm` or `view` event), `repeat_click` (3 or more clicks on the same control or cell within 2 s), `long_hover` (dwell >= 3000 ms), `refused_confirm` (with its `reason`), `error` (with the last user event before it), and `undo_churn` (3 or more `remove` / `undo` / `clear` plan edits for one unit in one round). Each flag names the nearest earlier note if one is within 30 s. The agent reads these flags against the code to judge whether the behaviour was intended; the report never fails the run (exit 0 unless the file is invalid).
13. Render (agent side): without `--headless`, under `xvfb-run -a -s "-screen 0 1280x720x24" godot --rendering-driver opengl3`, `--render=notes|rounds --out=<dir>` loads the arena in playback mode, jumps to each note (or each round end), waits two frames and saves `note_<n>.png` / `round_<r>.png`.
14. Typing and placement: strict static typing; rules engines `RefCounted` in `scripts/playtest/` (`playtest_record.gd`, `plan_codec.gd`, `round_outcome.gd`, `content_fingerprint.gd`, `playtest_recorder.gd`, `playtest_replayer.gd`, `playtest_sweep.gd`, `playtest_store.gd` for ids, paths, list, atomic save); nodes in `scripts/ui/playtest/` (`playtest_hub.gd`, `ui_trail.gd`, `note_box.gd`, `playtest_browser.gd`, `playback_banner.gd`). Everything is inert when `OS.is_debug_build()` is false.

## Acceptance checks (rule-derived; `playtest_test_suite.gd`, files under `TestHelpers.sandbox("playtest")`)

1. Codec: every `PlanStep` script property is covered by the codec table; for each mode built in a catalog encounter, encode, decode and encode again give equal dicts and `plan.entries_for` gives the same blocks and skills.
2. Round trip: a battle played through the session API with presets and recorded by `PlaytestRecorder` replays with every round `match` and the same `end`.
3. Redo: confirm round 2, `reset_round`, plan differently, confirm: the record has `attempts == 2`, one `discarded` plan, a `rng_state` different from the first attempt's, and replays as `match`.
4. Divergence: changing one recorded `hp` makes the replay report `diverge` at that round, unit and field; later rounds `unverified`; tool exit 1.
5. Staleness: changing one sandbox JSON file changes `total` and lists exactly that path as changed; replay still runs.
6. Validation: `version` 2 refused, unknown `mode` refused, ref index out of range refused, each with round, unit and step in the message; missing optional fields take the defaults; an `rng_state` above 2^53 survives save and load unchanged.
7. Take-over: replaying to round N leaves `current_round == N`, phase PLANNING and unit state equal to the recorded outcome of round N-1; the branch record has `parent` and rounds 1..N-1 equal to the source.
8. Variant: an identical patch gives no `DIFF` line; a changed patch gives at least one and leaves rounds before N unchanged.
9. Sweep: the recorded seed's row equals the plain replay's `end`; summary counts add up to the seed count; a seed list of one gives no outlier.
10. Notes and trail (RefCounted parts): a note stamp built from a fake context fills every G field, an empty image is allowed; the trail stops at 20000 events and counts `trail_dropped`; `key` events are skipped when a text field has focus.
11. Release guard: with the debug check forced off (test seam), the recorder writes no file.
12. Trail report: a synthetic trail with one unhandled click and no follow-up, one handled click followed by a `button` event, four clicks on one control in 1 s, a 3500 ms hover and an `error` event gives exactly one `dead_click`, one `repeat_click`, one `long_hover` and one `error` flag (the error names the click before it); the `Logger` hook records a `push_error` message as an `error` event while recording and nothing after recording stops.

## Out of scope

- Exact replay of overworld, region map, dialogue and Forge sessions: covered by the trail and notes only. Later phase PLAYTEST2 (idea): `GameState` snapshot at start + an `ExploreRunner` / `DialogueRunner` command log replayed on `GameRolls`, and `GameEncounter` battles with the saved party.
- Video or audio capture, uploading files anywhere, editing recordings in the game UI, a seed or config editor for the human, release builds.
- Any change to live combat behaviour (redo RNG, AI, FX timing); FX instant mode belongs to S14-L2.

## Open questions (recommended default first)

Answered 2026-10-17: the user approved the spec with every default below.

1. Recording on by default in debug runs? Default: yes, with a Scene Runner toggle (`record_playtests`).
2. Screenshot format: JPG quality 0.85 at viewport size (about 100-200 KB) or PNG (lossless, 3-5x bigger in the repo)? Default: JPG.
3. Sharing: Copy to repo into `playtests/inbox/` and push, or attach files to the chat by hand? Default: repo inbox; the planner removes handled inbox folders in a cleanup pass (git history keeps them).
4. Note tags: `mechanic`, `ui`, `ux`, `bug` only, or add `idea` / `balance`? Default: the four.
5. Should a live round redo re-use the same rolls (no reroll by redo)? This is a gameplay choice outside this task; default: unchanged (the recording handles either).
6. Hover dwell 700 ms and the F8 key: defaults as written.
