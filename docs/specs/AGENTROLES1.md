# AGENTROLES1: who owns what, and the integration agent

Status: approved (user 2026-11-04: all as recommended).
Depends on: none to start; LAYOUTDATA1 covers UI and is not re-specified here
Lane: design for this spec; slices in section 11
Touches: section 11

## 1. Recommendation

1. Add one role, `integration`. It plays the real game in a real muted window, measures it and reports. It fixes nothing.
2. Build one engine tool, `aegis scenario`: JSON scenario files walk real screens over the agent link and assert state.
3. Keep real windows out of the gate. The gate only parses the scenario files (under 1 s).
4. Perf gives "slow", "ok" or "no verdict", never a failed gate: this machine is too busy for hard numbers.
5. Give each kind of work one owner (section 8). Under decision "Agents stay in scope; gaps go to the main session", one-off
   work goes to the main session; a repeating gap goes to the user with a recommendation. Procedures that recur become skills.
6. Move proof work from builders in four steps, so nothing falls between (section 9).

## 2. What exists today

Read in code at the planner worktree's head, 2026-11-04. Nothing here was run.

| Check | Covers | Data | Real screens? | Key fact |
| --- | --- | --- | --- | --- |
| Journey golden | World loop: walk, talk, shop, travel, save | Frozen | No | "Fights are not played" (`journey.rs:6`); frozen data (`:57`) |
| `sim_golden` | Sim output lines | Frozen | No | Balance modes only |
| `feel` sim | AI fights, poise, AP | Scratch in its tests | No | `auto_driver.rs` compares host and sim on shipped `tl_` fights |
| Timeline golden | Timeline rules | Frozen | No | - |
| PHASEFOLD | Fights with a wave | Shipped fights, recorded once | No | `phasefold.rs:1-8` |
| Store resave, `content check` | Writer form, row validity | Shipped | No | Reachability checked inside one dialogue only (`check/dialogue.rs:621`) |
| GUI tests (482 `#[test]`) | One screen at a time | Debug game, art off in several (`region.rs:2217`) | One each | `Shell` is private (`shell.rs:42`), built only in its own tests (`:1638`) |
| Pane hashes | Six panes | Fixture | Pane only | `panes_native.rs` |
| `text_audit` | Text faults, one frame, top screen | Whatever is open | On request | `shell.rs:855`; no sweep |
| `shot_diff` | Two pictures of a screen | Files | Pure | `shot_diff.rs` |
| Agent link | Drive a live game | Live | Yes | `shell.rs:905-1108`; no wait or assert in `aegis link` (`aegis_cli/src/lib.rs:276`) |
| Trail report | Dead clicks, errors, asset fallbacks | A recording | After the fact | `trail.rs`; recording off in release by default (`screen.rs:127`) |
| `tools/fc.sh gate` | Build, tests, layers, docs, tool tests | - | No window | `fc.sh:194-206` |

Gaps:

| # | Gap | Evidence |
| --- | --- | --- |
| G1 | No test plays the game across screens on shipped data | Table above; no game test reaches `Shell` |
| G2 | No test takes a real fight through results and back to the map | Journey draws outcomes; sims have no screens |
| G3 | The shipped campaign has no loop test | Journey data is frozen, so `sunk_ward` cannot reach it. CAST1-A4's row lists `journey.rs`; worth a second look |
| G4 | Perf is measured only when a builder chooses | `redraw` has no timing (`shell.rs:1146-1176`); numbers sit in rows, no trend |
| G5 | Screens are checked one at a time | `text_audit` needs a screen open; TEXTFIT1 was by hand |
| G6 | Window-driving is rewritten each time | 20 scripts, 2,451 lines, in three lanes' ignored `.agent/`, on 7 hand-picked ports; timed by sleeps |
| G7 | Fields can validate and do nothing | The review pending from CAST1-A3 |
| G8 | Gate time has no trend | Scattered in `docs/EFFICIENCY.md`: 2 min idle, 873 s loaded |

## 3. The integration role

| Topic | Rule |
| --- | --- |
| Owns | Whole-game scenarios, real-window proof, whole-screen perf, `games/emberwake/docs/INTEGRATION_REPORT.md`, `docs/metrics/perf.jsonl` |
| Writes | `games/*/scenarios/*.json`, those two files, boards in `C:/dev/localgen/review/<task>/` |
| Never writes | `*.rs`, `data/**`, assets, tests, goldens, `tools/**`. No fixes. A missing link command is reported as a gap with the exact need |
| Runs | `tools/fc.sh scenario`; `aegis replay ... trail-report` on the run's recording; `content check`. Never the gate |
| Per flush | After a green flush: every active scenario at 720p. Perf recorded, verdict only if quiet. Target 3 minutes (assumed) |
| Per milestone | Before the release PR: all scenarios at 720p and 1080p, a text audit sweep at all four text sizes, perf with a verdict |
| Per task | Only rows marked `proof: window`, in the task's lane after the builder ends, before review (builders leave work uncommitted) |
| On request | The planner names a scenario or a question |
| Report | Under 40 lines: commit, pass or fail per scenario and size, perf table, findings with a bundle path and "route to: logic, asset, ui, design, balance" |

| Agent | Looks at | Answers |
| --- | --- | --- |
| `change-reviewer` | One diff, reading it | Is this change safe and within the rules? |
| `balance` | Sim numbers on `balance-ready` systems | Do the numbers meet the targets? |
| `integration` | The whole built game, running it | Does the game still work end to end, and how fast? |

Model as `balance` (assumed). Worktree `C:/dev/tactical-rpg-integ`, detached at `develop`,
with a warm release cache. One real window at a time.

## 4. The driver: `aegis scenario`

In Rust in `aegis_cli`, not Python: `python` on this PATH is a stub, it is engine capability any game can use, and it is
testable in the gate. It replaces the 20 scripts and what they repeat: own port, scratch profile, muted, stop only what it started.

Commands: `tools/fc.sh scenario <id|all> [size=1280x720] [out=<dir>] [profile=dev|release] [slow=<n>]` and
`aegis scenario check [<file>|all]`. A dev build is enough for proof; perf needs `release`. Files: `games/<id>/scenarios/<id>.json`.

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `id` | string | required | Equals the file stem |
| `title` | string | "" | - |
| `status` | `active`, `waiting` | `active` | `waiting` needs `why`; parsed, never run |
| `launch` | object | `{"screen": "menu"}` | `screen` is `menu`, `arena` or `runner`; optional `encounter`, `seed` |
| `sizes` | list of `WxH` | `["1280x720"]` | Each at least 1280 x 720; the run records the real `canvas` |
| `settings` | object | `{}` | Merged over the scratch profile; `mute: false` refused |
| `timeout_s` | int | 300 | 1-3600 |
| `steps` | list | required | Not empty; at most 500 after `repeat` |

The scratch profile: `mute` true, Windowed, `window_size`, `vsync` false, `record_playtests` true, not taking focus.

| Step | Keys | Does |
| --- | --- | --- |
| `click`, `hover` | `id` | The link request; a reply with `ok` false fails the step |
| `key` | `key`, `times` = 1 | Key presses |
| `type` | `text` | Into the field being edited |
| `drag` | `id`, `to` | Press, move, release |
| `back` | - | Back |
| `link` | `body` | Any other request: battle commands, screen commands |
| `wait` | `until`, `timeout_ms` = 10000 | Polls until true, times `slow`; a timeout fails |
| `assert` | `until` | True now or fail |
| `shot` | `name`, `max_w` = 1920 | Screenshot into the bundle |
| `audit` | `scales`, `fail` = `["cut", "edge", "overlap", "spill"]` | `text_audit` per scale |
| `perf` | `name`, `frames` = 300, `warm` = 60 | Section 5 |
| `repeat` | `times`, `steps`, `until` | A bounded loop, for walking and fight rounds |

A condition `until`: `{"path": "state.location", "eq": "warden_keep"}`, ops `eq`, `ne`, `gt`, `lt`, `has`, `absent`; or
`{"widget": "<id>"}`, or `{"all": [...]}`. Paths read the `ui_tree` reply; `"source": "game"` reads the `game` reply.

Rules, each with a test on a fake link in `aegis_cli/tests/scenario.rs` unless marked:

1. After each action the driver waits for two more frames (new `frame` count in `ui_tree`), never a sleep. The link replies
   before the frame is drawn (`shell.rs:1148-1154`), which is why the scripts sleep.
2. A failed step stops that size and writes a bundle: `result.json`, `fail.png`, `fail_tree.json`, `stderr.txt`, the trail report.
3. The port comes from the OS. Settings and saves go to a scratch folder; the real profile is never read or written.
4. The driver stops only the process it started, on pass, fail or timeout. A game that exits or panics mid-run fails with its stderr.
5. Exit codes: 0 pass, 1 a step failed, 2 could not start. An unknown step or key fails `check` with the file and step index.
6. Real run on the shipped game: a broken `id` in a scenario fails with a `fail.png` (mutation check, not in the gate).

Not in the first version: save fixtures (saves below the current version are refused, so they would rot), headless runs, pixel asserts.

## 5. Performance

Add per-frame counters in `App::redraw` (draw is `shell.frame()`, present is the copy plus `p.render()`, `shell.rs:1154-1173`)
and a link request `perf {reset}` replying p10, p50, p95, max and count, plus each screen's first frame.

| Number | Sizes | Budget (release, quiet) | Basis |
| --- | --- | --- | --- |
| Draw + present p50 | 720p, 1080p | 16.6 ms | 60 fps. The 15 ms draw at 1080p from the brief leaves about 1.6 ms |
| Draw + present p95 | 720p, 1080p | 25 ms | Assumed |
| Hitch, max outside a first frame | 720p, 1080p | 100 ms | Assumed |
| First frame of the region map | 720p, 1080p | 100 ms compose | `REGIONGROUND1_ENGINE.md` 4.3; 38-50 ms worst measured |
| Start to main menu | 720p | 5 s | Assumed; the scripts wait 3 s after connecting |
| Working set | 1080p | Recorded only | No basis yet |
| Gate seconds and test count | - | Recorded only | From the planner at each flush |

Checkpoints: main menu, region map (Warden Keep, tilted, zoom 1 and 3), battle planning, battle playback. 1440p only if it fits
the screen, since a size is capped to the usable area; the real `canvas` goes in the record.

What is a realistic signal? `docs/EFFICIENCY.md` shows load hits totals much harder than a best case: a gate 2 vs 7.5 min, while
10 compose runs under load span only 21-25 and 38-50 ms (blit 2.4-4.2 ms). So:

1. Compare p10, not the mean: the fast frames are the ones load did not touch.
2. Time a fixed CPU workload (`calib`) before and after each checkpoint.
3. `no verdict` if `calib` is over 1.3x its baseline or the two differ by over 20 %. The record stays; only the verdict is withheld.
4. `slow` if p10 is at least 25 % and 2 ms over the median of the last 5 accepted records of that scenario, checkpoint and size,
   on two runs in a row, or once on a quiet re-run.
5. The commit range between the last good and first slow record goes in the report; the planner routes it.
6. Budgets count only on a quiet milestone run. A miss opens a row; it blocks nothing without the user.

All thresholds are assumed and re-tuned after 10 records. Dev-build times are never recorded.

One line per checkpoint in `docs/metrics/perf.jsonl`: `date, commit, scenario, checkpoint, size, canvas, frames, p10, p50, p95, max,
first, calib_before, calib_after, verdict`.

## 6. First scenarios

| Id | Status | Walks | Asserts |
| --- | --- | --- | --- |
| `warden_keep_smoke` | Active once built | `menu.new_game`, `new_game.start`, the region map, walk to a marker, talk, game menu, journal, save, back to menu, load | `state.location` is `warden_keep` (`ashlands/world.json:8`), each `stack`, the save reloads, no error or asset fallback in the trail |
| `tl_flank_fight` | Active once built | `screen=runner`, pick `runner.arena.encounter` and `runner.arena.ruleset`, plan a round, autoplay to the end, results, back | `screen` is `battle`, an outcome in the battle state, `battle_results` appears, perf at planning and playback |
| `sunk_ward_slice` | `waiting` | New Game, the prologue, the slice nodes and fights, to `slice_end` | Each story beat, fight result and campaign meter from the data |

`sunk_ward_slice` waits for the data: `start_party` is empty today (`sunk_ward/world.json:9`), and it needs CAST1-A4 to A6,
SUNKWARD1, CAMPAIGN1-D, ROSTER1-D and STORYUI1-A. Its beat list comes from STORYUI1 and CAST1 when it is written.

The region map has no link commands of its own: only `battle.rs:1076` overrides `command`, and `rv_render.py` carries its own route
finder. INTEG1-G adds `walk_to {marker | cell}` over the existing `walk_to_marker` and `path_to` (`region.rs:634`, `:590`).

Unverified: whether `autoplay` over the link in a GUI battle plays the fight on screen or jumps to the end. INTEG1-S1 finds out.

## 7. What the gate can't hold

| Limit | Fact | Rule |
| --- | --- | --- |
| CI's Linux box | No display step; only ALSA headers installed (`gate.yml`); `pixels` needs a GPU surface | No window on CI |
| Pictures differ by platform | CIPANES1: one pane's hash differed on Linux, from an OS-worded error text | Scenarios assert state, not pixels |
| Desktop gate time | About 5 min, up to 873 s loaded; the brief saw 25 min | Nothing that opens a window goes in `fc.sh gate` |
| Wall-clock timers | The battle runs on `Instant::now` (`battle.rs:402`, `playback.rs:257`) | A headless run still waits real time, so it would not save much |
| User at the keyboard | A window can take focus | Scratch profile starts inactive; the planner holds runs if the user asks |
| Busy machine | Lanes compile at the same time | Runs after a flush; never two at once |

In the gate: `aegis scenario check all` as a test (`emberwake_host/tests/scenarios.rs`) and the fake-link tests, under 1 s together.
A headless `Shell` stays later: only if a scenario catches something a window run misses by too long.

## 8. Scope coverage

Four answers: an **agent**, the **main session**, a **skill** (invoked by either), or **user** judgement. Gap types: **one-off** goes to the main
session; **repeating** goes to the user with the recommendation shown, which is not yet approved. Roles today: planner, `design-brainstorm`,
`logic-builder`, `asset-builder`, `balance`, `change-reviewer`, `housekeeper`, plus `ui-author` (approved in LAYOUTDATA1). `integration` is the only addition wanted.

| Work | Owner today | Proposed owner | Change | Gap, evidence |
| --- | --- | --- | --- | --- |
| Queue, decisions, commits, merges | Planner | Main session | None | - |
| Options, specs, art prompt text | Design; planner sometimes | Design | None | - |
| Behaviour code, small widgets, schema | Logic | Logic | None | - |
| Unit and rule tests, goldens for own change | Logic | Logic | None | - |
| Tool code (`tools/**`) | Logic or planner | Logic | Row says so | Repeating: 23 rows name `tools/sprites`, `tiles` or `portraits`. Rec: logic |
| Running tile, prop and rig tools | Mixed: DASH1 row says asset, a logic builder did it; PROPS1-A2 "logic, then asset; asset" | Rec: asset | Split rows | Repeating, same 23. User decides |
| Data rows | Asset | Asset | None | - |
| Placing art files in `assets/` | Asset by row (LOOK1-R), but its brief says ONLY `data/**` | Rec: asset | Widen brief | Repeating: LOOK1-R, PROPS1-A2, others. User decides |
| Art batches: queue, ingest, boards | Main session (decision "Art goes straight to develop") | Rec: asset, in the planner worktree; the user relay stays in the main session | Decision change | Repeating: 9 `art` rows, no agent. User decides |
| Mock and comparison boards | Main session, builders | Skill `review-board`; screens by `ui-author` | New skill | Repeating: 78 board PNGs |
| Screen layouts | Logic | `ui-author` | LAYOUTDATA1 | Decided |
| Real-window proof, render scripts | Builders | `integration` | Section 9 | Wanted: G6 |
| Whole-game and cross-screen checks | Nobody | `integration` | New | Wanted: G1, G2, G5 |
| Whole-screen perf | Builders, when they choose | `integration` | New | Wanted: G4 |
| Budget on one module named in a spec | Logic | Logic | None | - |
| Partial gate | Builder | Builder | Skill note | Overlap |
| Full gate | Reviewer, planner | Main session at flush; reviewer reads the log | Reviewer brief | Overlap: the reviewer brief says run it; EFFICIENCY says skip a fresh log |
| Diff review | `change-reviewer` | `change-reviewer` | Shorter | - |
| Fix found in review | Planner inline or same builder | Rec: same builder, medium effort | Q5 | Repeating, by decision "Logic lane scope". User decides |
| Small standalone logic | Planner, by decision | Main session if one-off; reviewer still reviews | Q5 | Stands until the user says otherwise |
| Sims and report | `balance` | `balance` | Brief | Brief says `main`; the last report measured `develop` a39ccb56 |
| Balance tool needs (`tl_`, feel keys) | Logic, from report | Logic row, from the report's list | None | Repeating: RULESET1-BX, -G2, PACE1-S. Works as is |
| Applying approved numbers | Asset | Asset | None | - |
| Cleanup proposals | `housekeeper` | `housekeeper` | None | - |
| Docs delta | Builder writes, planner applies | Same, as a clean hand-off | `task-landing` gains it | Not a gap |
| README, brief, decisions, HISTORY | Planner | Main session | None | - |
| Patch notes, release PR | Planner (skill names the cloud thread) | Main session | None | - |
| CI failure reading | Main session | Skill `ci-read` | New skill | Repeating: see below |
| Launch for the user | Main session | `pc-launch` | Extend | Repeating: see below |
| Feel calls | User | User; tools by logic | None | - |
| Other: seed cleanup (CLN-49), ARTPROMPT1, CAMP1-P | Planner | Main session | None | One-off |

### Rule for a task that crosses scopes

The planner applies this at dispatch. At least 20 rows now carry two lanes in one cell ("logic, then asset").

1. List what the task changes and runs. Sort each by what it is, not by who is free:

   | It is | Owner |
   | --- | --- |
   | Behaviour, or a tool's code | Logic |
   | Values or files read at runtime, no code; running an existing tool | Asset (screen layouts: `ui-author`) |
   | Checking the whole game or a real window | Integration |
   | A verdict on numbers | Balance |
   | A design choice | Design, then the user |
   | Only the main session can do it (user loop, merges, records) | Main session |

2. One owner: dispatch. More than one: split into a row per owner, in the order code, data, proof, each with one lane value.
3. Do not split what a role's brief already includes: a schema slice's re-save, test fixtures, a module's own timing, a docs delta.
4. An agent that meets out-of-scope work mid-run stops and reports the exact need, as with a schema gap. It does not do it.
5. `change-reviewer` item 5 checks the diff's paths against the row's one lane. DASH1's `tools/` change on an `asset` row would fail.
6. Something fitting no row, once: main session. The second time: the planner tells the user (decision "Agents stay in scope").

### Skills: procedures that recur

Rough counts: command mentions in the planner's session log and files on disk, 2026-11-04. Ranked by repeats saved.

| Rank | Procedure | Evidence | Answer | Invoked by |
| --- | --- | --- | --- | --- |
| 1 | Land a task with its message | `stage.sh` 43 mentions; 31 `msg_*` and 30 `land_*` files; `landtask.sh` is in no skill | Extend `task-landing`: `landtask.sh` first, message format | Main session |
| 2 | Apply a docs delta and as-built row | 80 of 362 commits since 2026-10-28 say "as built" | Extend `task-landing` | Main session |
| 3 | Real muted window on its own port | 20 scripts, 2,451 lines | New `real-window` now; a pointer to `fc.sh scenario` after INTEG1-E1 | Integration; builders until cut-over |
| 4 | Mock or comparison board | 78 board PNGs; 5+ board and 8 mock scripts in scratch | New `review-board`: Pillow's full path, scale only, folder, links, repeated-area rule | Main session, `ui-author`, asset |
| 5 | Check a test fails when the rule breaks | 19 mutation files in lanes | New short `mutation-check` | Logic |
| 6 | Which gate for this change | `pgate.sh` 15 mentions; known-issues row on kit changes | A table in `engine-checklist` | Builders, reviewer |
| 7 | Read a CI failure | `gh run` 17 mentions; CIPANES1 | New short `ci-read`: run, failed log, Linux vs Windows, known-issues | Main session |
| 8 | Launch a build from a copied exe | 6 mentions | Extend `pc-launch` (a running game locks the exe) | Main session |

Not skills: seeds and multi-site edits (`patch-and-seed`), the docs a change needs (`task-landing`). Three new skills, one of them short-lived.

**Planner scripts.** `lane.sh`, `stage.sh`, `flush.sh`, `cleanup.sh`, `pgate.sh`, `land.sh`, `landlane.sh` and `landtask.sh` live in
`C:/Users/tusha/.claude/projects/c--dev-tactical-rpg/`, beside the memory folder, untracked. Briefs call `<planner dir>/pgate.sh`, so a
builder's last step depends on them, and there is no review or history. They belong in the repo: `tools/planner/`, paths from one
env file with today's as defaults. Per-run `msg_*` and `land_*` files stay out. `land.sh` looks replaced by `landlane.sh`; check when moving.

## 9. Handover to the integration agent

| Responsibility | Today | After cut-over |
| --- | --- | --- |
| Real-window proof of a task | Builder's script | Integration, on rows with `proof: window` |
| Render scripts and boards from real runs | Builder | Integration, as scenario `shot` steps |
| Whole-screen timing | Builder when it chooses | Integration |
| Text audit at sizes across screens | Builder or a task | Integration |
| Cross-screen checks | Nobody | Integration |

Builders keep: unit and rule tests, mutation checks, the partial gate, new paths tested on real data in a test (self-check item 4,
minus "what was viewed"), a timing budget their spec sets on their own module (`ground_compose --timing`), the docs delta. The user's board reviews do not change.

Brief changes:

| File | Change | Length |
| --- | --- | --- |
| `logic-builder.md` | Add: no window scripts or whole-screen timing, state ids a proof needs. Finish with the partial gate, not "`check_project.ps1`" | Shorter |
| `change-reviewer.md` | Item 1: read a gate log newer than the changes, else partial gate; drop "what was viewed"; read the integration line on `proof: window` rows; item 5 per the rule above | Shorter |
| `asset-builder.md` | Drop "manifests not writable" (`content world`, `content manifest` exist). Add per Q6. Finish with `content check` and the partial gate | About equal |
| `balance.md` | Branch per Q7 | Equal |
| `integration.md` | New, from section 3 | New |
| `design-brainstorm.md`, `housekeeper.md` | None | - |

Sequence:

| Step | Starts when | Builders | Integration | Moves on when |
| --- | --- | --- | --- | --- |
| 0 | Now | As today; `real-window` skill replaces new one-off scripts | - | INTEG1-E1 and -G land |
| 1 | Driver landed | Proof through `fc.sh scenario`, no new scripts | Writes first scenarios | S1 passes on `develop` |
| 2 Shadow | S1 passed | Still prove their own work | Runs at each flush, and on two `proof: window` tasks | Three flushes, at most one false failure, every builder finding also found |
| 3 Cut-over | Shadow passed, user told | Briefs change; proof stops | Owns the table above | - |
| 4 Verdicts | 10 records | - | Perf flags count | - |

Until step 3 builders own proof. From step 3 a `proof: window` row cannot land without an integration line, and if the agent is blocked the row
waits, so a missing proof shows. If the shadow fails, step 2 repeats and builders keep proof.

## 10. The wider split

Sampled, not crawled. UI is LAYOUTDATA1.

| Aspect | A data or design agent alone today? | Hard-coded, sampled | Engine work first? |
| --- | --- | --- | --- |
| Skills, passives, items, classes, units | Yes: CAST1-A2 no logic row, A3 one | Effect vocabulary closed (`trigger.rs:664-772`), which is fine | No |
| Statuses | No | The table is Rust (`rpgkit_rules/src/status.rs:55-66`). This caused EMPOWER1 | STATUSROWS1, at the next gap |
| Encounters, hazards, meters, zones, rulesets | Yes | Ruleset `Numbers` are rows (`ruleset.rs:292`) | No |
| Core formulas | Numbers no | `damage.rs:12-19`, `stats.rs:6-9`, `budget.rs:11-23` | Only when balance asks for a value |
| AI | Per-unit yes (`AiProfile`, `world.rs:99`) | Thresholds `ai.rs:20-28`; `Role`, `Tactic`, `UnitTactic` closed. 4,400 lines of planner | New behaviour stays logic work |
| Story, quests, campaign | Yes, on STORYUI1, CAMPAIGN1 rows | Objective kinds closed | No |
| Maps, grounds, props | Yes | - | No |
| Feel timings | Yes: `tuning.json` reloads live | Walk and run steps in `mapview.rs:50-52` | Small, with a feel ask |
| Costs | Partly | `progression.rs:264-265`, `shop.rs:14-18` | At first need |

Rule: a constant moves to data when a second value is asked for, not ahead of time. A move must leave every golden unchanged.
Lanes become: plan, design, logic, asset, `ui` (LAYOUTDATA1), balance, review, cleanup, `integration`. One lane value per row.

## 11. Measure and slices

"Kit mostly done" (decision "Engine goal"), tracked in the `milestone-status` skill:

| Number | How | Today |
| --- | --- | --- |
| Open logic rows per milestone | `TASKS.md`, lane `logic`, not done or dropped | About 48 in all milestones and backlog, by a rough parse |
| Logic rows per content batch | Rows saying "Created by `<batch>`", as EMPOWER1 does | CAST1-A2: 0. CAST1-A3: 1, review pending on dead fields |
| Binding or schema gaps per `ui` and asset run | From reports | 0 `ui` runs yet |

Assumed target: the last 3 batches made at most 1 logic row each, none a new system, and the next milestone opens with at most 5 logic rows.

| Id | Title | Layer | Lane | Depends | Touches | Runs |
| --- | --- | --- | --- | --- | --- | --- |
| INTEG1-E1 | `aegis scenario` run and check, `frame` in `ui_tree`, start inactive, `fc.sh scenario`, authoring page | engine | logic | - | `aegis_cli/src/{lib,scenario}.rs`, `aegis_cli/tests/scenario.rs`, `aegis_ui/src/shell.rs`, `tools/fc.sh`, `engine/docs/authoring/scenarios.md` | 1 |
| INTEG1-G | Region map `walk_to`; the parse test on shipped scenarios | game | logic | E1 | `emberwake_gui/src/region.rs`, `emberwake_host/tests/scenarios.rs` | 0.25 |
| INTEG1-E2 | Frame counters, `perf` request and step, `calib` | engine | logic | E1 | `aegis_ui/src/{perf,shell}.rs`, `aegis_cli/src/scenario.rs` | 0.5 |
| INTEG1-E3 | `perf.jsonl`, `aegis scenario perf-report`, verdict rule | engine | logic | E2 | `aegis_cli/src/scenario.rs`, `docs/metrics/perf.jsonl` | 0.5 |
| INTEG1-R | Role note, lane, worktree, report file | process | planner | E1 | `.claude/agents/integration.md`, `CLAUDE.md`, planner skill | - |
| INTEG1-S1 | `warden_keep_smoke`, `tl_flank_fight` | game | integration | E1, G, R | `games/emberwake/scenarios/` | 1 |
| INTEG1-S2 | `sunk_ward_slice` | game | integration | S1, STORYUI1-A, CAST1-A6, CAMPAIGN1-D, ROSTER1-D | `games/emberwake/scenarios/` | 1 |
| INTEG1-X | Cut-over briefs after the shadow | process | planner | S1, three flushes | `.claude/agents/*.md`, planner skill | - |
| SCOPE1 | Brief fixes needing no integration; one lane value in new rows | process | planner | user answers | `.claude/agents/*.md`, planner skill | - |
| SKILLS1 | Scripts to `tools/planner/`; the skill table | process | planner | - | `tools/planner/`, `.claude/skills/**` | - |
| KITDONE1 | The measure in `milestone-status` | process | planner | - | `.claude/skills/milestone-status/` | - |
| STATUSROWS1 | Status table as rows, goldens unchanged | kit | logic | Next status gap, own spec | `rpgkit_rules/src/status.rs`, schema | 1 |

Rough total for integration: 2.25 logic runs, 2 integration runs, planner edits. **Start now, no wait on LAYOUTDATA1:** E1, then G and E2;
SKILLS1 and KITDONE1 at any time. Overlap: E1 and LAYOUTDATA1-D2 both change `ui_tree` in `shell.rs`, so not in parallel; D2 waits on E5 anyway.

## 12. Verified and assumed

Verified in code or files: every file:line above; script and board counts on disk; TASKS counts by rough parse on 2026-11-04.
Rough: skill counts come from a pattern search of commands in the session log, so treat them as order of magnitude.

Assumed or not run: all perf thresholds, run times and targets; that a window can start inactive and keep drawing while covered (E1 tests it);
autoplay over the link (section 6). From the brief, not found in the docs: the 15 ms draw at 1080p and the 25-minute gate.
That a logic builder did DASH1 comes from the user's message; its row shows only the lane and the tool files.
No HISTORY row touches a scenario driver or integration agent; the retired items are Godot parity suites (`docs/HISTORY.md` lines 129, 132), a different job.

## 13. Open questions

1. **Cadence?** Recommend scenarios per flush, full perf and sweep per milestone, proofs per marked row.
2. **Perf target?** Recommend 60 fps at 1080p on this laptop, quiet, as a flag that blocks nothing.
3. **What can a failure block?** Recommend a scenario failure blocks the release to `main`, not a flush; a perf flag blocks nothing.
4. **The window during runs.** Recommend it starts inactive, and the planner holds runs while you play.
5. **Fixes between review and landing.** Decision "Logic lane scope" has the planner do them; its reason was builder cost. Recommend: review fixes go to the
   same builder at medium effort (decision "Builder efficiency"); standalone small logic stays one-off main-session work. This narrows a settled row, so it is your call.
6. **Repeating gaps.** Recommend asset-builder takes (a) art batches in the planner worktree with the user relay kept in the main session, and (b) art placement and running existing tools; (c) tool code stays logic; (d) boards become a skill. No new agent type.
7. **Balance branch.** Brief says `main`; the last report measured `develop`. Recommend `develop`: a release is late for a verdict.
8. **Who writes a task's proof scenario?** Recommend integration, from the ids the builder lists.
9. **STATUSROWS1 now or at the next gap?** Recommend at the next gap; EMPOWER1 is enough today.
10. **Skills.** Recommend the eight in section 8 and moving the scripts into the repo.
