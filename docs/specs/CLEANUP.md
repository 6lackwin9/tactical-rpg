# CLEANUP: housekeeping proposals (housekeeper agent)

Newest pass on top. Candidate statuses: `proposed`, `confirmed (user <date>)`, `rejected (user <date>): <reason>`, `done`. Only the user confirms; confirmed candidates become `docs/TASKS.md` rows. Cleanup is never folded into a feature task: "after X" = run once X is merged, "superseded by X" = X deletes it anyway.

## Pass 2: docs audit (2026-10-29, develop at 4b8f6b1)

Scope (user 2026-10-29): docs whose status went stale because tests cover only code (part A), docs that no longer
serve a purpose (part B), and cheap checks that catch both after each task (part C). Read-only scan of the planner
checkout (detached `origin/develop`), `git worktree list`, `git log`, `progress.py --all` and `tools/fc.sh docs`.
Every candidate is docs-only except CLN-33 (a tool); none touches code, data or tests. Lanes as in Pass 1: TASKS,
ROADMAP, MILESTONES, decisions, HISTORY, AGENT_MAP and `.claude/*` are the planner's; spec status lines and archive moves
may go to design-brainstorm. Archive moves follow CLN-5: the file keeps its text, gains a first line
`Archived <date>: <reason>; superseded by <x>`, and gets a row in the archive README.

### Pass 2 ranked summary

| Rank | Id | Part | What | Size | Lane | When | Confidence | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | CLN-19 | A | Move 97 done and 2 superseded rows out of TASKS; drop the stale BAL5-R1b duplicate | S | planner | at the next flush | high | done (2026-10-29) |
| 2 | CLN-20 | A | TASKS statuses vs live work: 3 running tasks read `approved`, 2 spec rows unfinished, 1 stale worktree | S | planner | now | high | done (2026-10-29) |
| 3 | CLN-22 | A | ROADMAP system rows: 13 statuses behind their tasks, answered questions still "open", FX1 missing | S | planner | now | high | done (2026-10-29) |
| 4 | CLN-33 | C | `tools/docs_status.awk` in `fc.sh docs`: status consistency checks (build first) | S-M | logic-builder | after CLN-19, CLN-22 | high | confirmed |
| 5 | CLN-27 | A | GAME_BRIEF: Godot-file and pre-KIT1 "where to look" pointers; split | M | design-brainstorm | now | high | confirmed |
| 6 | CLN-23 | A | Spec Status lines: 21 `proposed` specs approved since 2026-10-25, 40 built specs still `approved` | S-M | design-brainstorm | now (or with CLN-30) | high | confirmed |
| 7 | CLN-26 | A+B | Workflow docs: desktop worktree and branch steps, WORKFLOW.md loop and run log | S | planner | now | high | done (2026-10-29) |
| 8 | CLN-25 | A+B | AGENT_MAP: finished-run log stops at SPLIT1; archive the cloud-era log | S | planner | now | high (facts), question (log) | done (2026-10-29) |
| 9 | CLN-24 | A | Missing As-built notes (HOOKS1, CAMPAIGN1-C) and an open user pick buried in a done row | S | planner | now | medium | done (2026-10-29) |
| 10 | CLN-21 | A | Open task rows' Touches name files that moved to `kits/rpg` in KIT1 | S | planner | now | high | done (2026-10-29) |
| 11 | CLN-28 | A | MILESTONES: recent art tasks and ~45 pre-slice queued rows match no milestone | S | planner + user | now | high (facts), question (placement) | done (2026-10-29) |
| 12 | CLN-34 | C | `docs_needed.py` and task-landing: ROADMAP, spec status, As-built and done-row reminders | S | planner | with CLN-33 | high | confirmed |
| 13 | CLN-30 | B | Archive 52 built Godot-era game specs | M | design-brainstorm | after CLN-23 | medium-high | confirmed |
| 14 | CLN-31 | B | Archive PURSE1.md (superseded by REWARD1) | S | design-brainstorm | with CLN-30 | high | confirmed |
| 15 | CLN-32 | B | Archive the cloud handover files and CLEANUP Pass 1 | S | planner | now | medium-high | done (2026-10-29) |
| 16 | CLN-35 | C | Planner-side worktree check: live worktree vs row status | S | planner | with CLN-34 | medium | confirmed |

Questions for the user (not proposed, see "Pass 2 questions"): Q-A goals G1-G6 vs the slice milestones, Q-B the
pre-slice queued rows, Q-C CLN-9's two statuses, Q-D `docs/RPG_KIT_CATALOG.md`, Q-E LOCALGEN3 and ART2, Q-F the
finished-run log on desktop.

### CLN-19

Done rows left in `docs/TASKS.md`. Part A. Status: proposed. Confidence: high.

- File: `docs/TASKS.md`. Problem: 97 rows read `done` (49 from 2026-10-25, the rest 10-20 to 10-29: KIT1-*, DERIVE1-*,
  STAGE1-*, FX1-*, HOOKS1-S/F/G/GF/Z/E/K/X/M/C, RULESET1-R/S/T/IN/PL/P/K/V/AI/D/E/A, CAMPAIGN1-E/S/M/C/T, STORYUI1-E/S/W/U1,
  ENCOUNTER2-T/H, CAST1, CAST1-S/S2/A1/A7/R, PORTRAITFX1-E/S/T, DIR4-G/M/R1, ART3/3B/3C, LOCALGEN1/2, PIXEL2/3, the design
  parent rows, ...), plus PURSE1-A and PURSE1-B (`superseded by REWARD1-A/-B`, lines 49-50). The header (line 5) and
  the planner skill step 6 say done rows move to `docs/archive/TASKS_DONE.md`; nothing has moved since 2026-10-25.
  TASKS is 110 KB (the docs check warns at 25 KB) and half of it is history.
- Problem 2: `BAL5-R1b` (TASKS.md:27, `approved`) is also in TASKS_DONE.md:9 as `done`; `progress.py` lets the TASKS row
  win, so milestone-status lists a finished task as "ready to start".
- Fix: at a flush point (no landing in flight, so `resolve_docs.py` has nothing to merge), move every `done`,
  `superseded` and `closed` row unchanged to the end of TASKS_DONE.md as one batch; delete the TASKS copy of BAL5-R1b
  (the archive row stays). TASKS_DONE has no Layer column: drop the cell or add the column (planner's pick).
- Risk: none for tools (`progress.py` reads both files). Touches: `docs/TASKS.md`, `docs/archive/TASKS_DONE.md`.

### CLN-20

Task statuses behind the live work. Part A. Status: proposed. Confidence: high (checked 2026-10-29 01:00).

- `docs/TASKS.md:148` HOOKS1-H, `:231` RULESET1-G, `:110` WINDUP1-R1 read `approved`, but their worktrees are live with
  builder changes (`tactical-rpg-hooks1h`: 5 modified `rpgkit_battle` files; `tactical-rpg-ruleset1g`: commit d45c8a2c
  "RULESET1-G slice 1"; `tactical-rpg-windup1r1`: modified clips). Fix: `in-progress (branch claude/<id>)`, as the
  header asks, so the other environment and milestone-status see them.
- `:107` WINDUP1 and `:113` STATUSFX1 (design parents) read `approved (...; needs a spec)`; both specs were approved
  2026-10-28 (WINDUP1.md and STATUSFX1.md status updates) and their E/L rows are done. Fix: `done (spec written
  2026-10-28)`, like the FX1 and HOOKS1 parents.
- Worktree `C:/dev/tactical-rpg-windup1l` (branch `claude/windup1l`) is left over: WINDUP1-L landed (2aca58e8 is on
  develop). Fix: `tools/wt_remove.ps1 ../tactical-rpg-windup1l claude/windup1l`. `tactical-rpg-dev` is `run_dev.bat`'s
  standing worktree (desktop.md:46): keep, and keep its untracked playtest inbox.
- `:92` DIR4-R2 Depends reads "DIR4-R1 user sign-off (...)": the sign-off happened (DIR4-R1 done 2026-10-28), so the row is
  ready; reword to `DIR4-R1 (done)`. `:110` WINDUP1-R1 "after DIR4-R1F if that runs" is moot now that R1 runs first.
- Touches: `docs/TASKS.md`; the worktree removal.

### CLN-21

Touches of open rows name files that moved in KIT1. Part A. Status: proposed. Confidence: high.

- Problem: since KIT1-M/N the `emberwake_{data,rules,battle,content,world,sim}` crates are façades (`src/lib.rs` only, 2
  to 38 lines); the code is in `kits/rpg/crates/rpgkit_*`. KIT1-P re-pointed the specs, not the TASKS rows, so the
  overlap check compares paths that do not exist. Rows: SKILLTEXT2 (`emberwake_world/src/skill_text.rs`, now
  `rpgkit_world`), SIMKPI1 (`emberwake_sim/src/roles.rs`), REWARDXW1 and ROSTER1-L (`emberwake_world/src/{battle,state,
  save,progression,dialogue,quest,data}.rs`), ROSTER1-T, -P, -K, -M, -B, LEVERS1-L1/-L2/-L3 (`emberwake_battle/src/
  {skill,plan,ai,command}.rs`, `emberwake_rules/src/status.rs`), MARKERS1 (`emberwake_content/src/art.rs`),
  CAMPAIGN1-SIM, -TM, -P, ENCOUNTER2-HA and RULESET1-G (`emberwake_sim/src/args.rs`, `emberwake_battle/src/dojo.rs`).
  New files a row will create (`roster.rs`, `runover.rs`, `board.rs`) are fine.
- Fix: re-point each Touches cell to the `kits/rpg/crates/rpgkit_*` path (the KIT1-P map). Touches: `docs/TASKS.md`.

### CLN-22

ROADMAP system rows behind their tasks. Part A. Status: proposed. Confidence: high.

- File: `games/emberwake/docs/ROADMAP.md`.
- `:32` DERIVE1 `proposed`, notes "DQ1-DQ5 open": all five rows done 2026-10-25; DQ1-DQ5 decided (`docs/specs/DERIVE1.md:286`). Fix: `done (2026-10-25)`.
- `:34` RULESET1 `proposed`, "TQ1-TQ11 open": 12 rows done, TQ answered (`decisions/lore.md:51` "Spec questions RULESET1
  and ENCOUNTER2"). Fix: `in-progress`. `:33` ENCOUNTER2 `proposed` (T, H done): `in-progress`.
- `:37` WINDUP1 `approved` (E, L done, R1 running), `:38` STATUSFX1 `approved` (E done), `:43` PORTRAITFX1 `approved`
  (E, S, T done), `:116` DIR4 `approved` (E, S, G, M, R1 done), `:114` PROPS1 `approved` (A2 left): `in-progress`.
- `:117` STAGE1 `approved` (all six rows done, reviewed 2026-10-27), `:112` ASSETORG2 `approved` (6 rows in TASKS_DONE),
  `:113` PLAYUI1 `approved` (9 rows done), `:118` REWARD1 "in-progress (... B next)" (REWARD1-B done, 23e65751), `:100` TRAV2
  `approved` (TRAV2-L, -A done): `done`.
- FX1 has no system row although M1 counts six done FX1 rows: add `FX1 | in-progress (FX1-A left)`.
- `:30` S16 `approved` while PLAYTEST1 and PORT-PLAYTEST are done: `done for battles; PLAYTEST2 idea`.
- Godot pointers in notes: `:15` S1 "`DamageCalculator` is the only formula" (now `rpgkit_rules::damage`), `:29` S15
  `tools/balance_sim.gd` (now `aegis sim`, `rpgkit_sim`), `:16` S2 "BAL2-R rerun and BAL3 next" (both done).
- G6 (`:56`) names CLN-9, CLN-17, CLN-3b as its finish line: CLN-17 closed as obsolete, CLN-3b done, CLN-9 see Q-C.
- Touches: `games/emberwake/docs/ROADMAP.md`.

### CLN-23

Spec Status lines. Part A. Status: proposed. Confidence: high.

- Fix pattern (as the 2026-10-20 sweep did): a new first line
  `Status update (2026-10-29): <status>; <tasks>. The status line below is the original.`; never rewrite the old line.
- `proposed` but approved by the user 2026-10-25 ("build the vertical slice") and partly built: `docs/specs/`
  CAMPAIGN1 (+_TASKS), ENCOUNTER2 (+_TASKS), HOOKS1 (+_TASKS), ROSTER1 (+_TASKS), RULESET1 (+_INTENT, +_TASKS),
  STORYUI1 (+_TASKS), PORTRAITFX1 (+_TASKS); `games/emberwake/docs/specs/` CAST1 and its seven parts (ASSETS, BESTIARY,
  BOSSES, ENCOUNTERS, FOUNDERS, GAPS, GEAR), FX1 (+_TASKS; all but FX1-A built), PROPS1 (answered all A 2026-10-22,
  five of six rows done).
- `proposed`/`approved` but every task done: KIT1, KIT1_TASKS, DERIVE1, ASSETORG2, SPLIT1, RESCOPE1 (`docs/specs/`);
  PLAYUI1, SPRITE1, REWARD1, SKILLUI1, FOUNDER_PORTRAITS (except FOUNDERPORT1-P) and the Godot-era built specs AUTO1,
  DISPLAY1, COMBAT_UI2, BAL3, REVIEW_F, PHASEFOLD, ART1, CAMP1 (except CAMP1-P), TRAV1_A/L, TRAV2_A/L, REP1_A/L, UI1_A/L,
  UI2_L, UI3_L, S12C_A/L, S12S_A, S12T_A/L, S14_A1, S14_A2, S14_L3, FORGE1_CLASSES, FORGE1_ITEMS, INTENT1 and BREPORT1
  (their PORT rows done), MUSIC1 (no Status line; MUSIC1-L/-A done). `spec`/`review` but done: BAL2_SIM, S14_L2,
  PLAYTEST1_L. DIR4 `approved`: in progress.
- Specs that CLN-30 archives get the update in their archive line instead. Touches: the listed spec files.

### CLN-24

As-built notes and a buried open question. Part A. Status: proposed. Confidence: medium.

- `docs/specs/HOOKS1.md` and `HOOKS1_TASKS.md` have no "As built" note, while the HOOKS1-E row (TASKS.md:144) records
  user-confirmed rulings the spec does not state: a redirect's leftover counts against the holder, ripostes raise no
  hit events, area holders take no redirect, the depth-1 rule; and an open pick: "`ally_hit` non-hit effects
  (Intercepting Hook's pull) run after the action, pending the user's pick". The pick is in no decision row and not in
  `games/emberwake/docs/lore/OPEN_QUESTIONS.md`. HOOKS1-K/-X/-M rulings are in decision rows (combat.md:41, :43), but the
  spec does not point to them.
- `docs/specs/CAMPAIGN1.md`: CAMPAIGN1-C's "the retreat falls back to the wake when the hub cannot be entered"
  (TASKS.md:170) is not in the spec.
- Fix: an "As built" section per task in HOOKS1.md (E, K, X, M, GF, Z, C; one line each plus the decision row) and
  CAMPAIGN1.md (C); the planner asks the user the `ally_hit` timing pick and records it in `decisions/combat.md`.
- Touches: `docs/specs/HOOKS1.md`, `docs/specs/CAMPAIGN1.md`, `games/emberwake/docs/decisions/combat.md`.

### CLN-25

AGENT_MAP. Part A and B. Status: proposed. Confidence: high (facts); the log question is Q-F.

- `docs/AGENT_MAP.md:15` "Finished, by milestone": the newest section is SPLIT1 (2026-10-19); nothing for M0 (released,
  PR #50) or M1, though planner skill step 3 asks for it. Lines 11 and 13 of "Active now" are 2026-10-19 and 2026-10-17
  states; line 7 is the 2026-10-28 wind-down while three builders run today. 1294 lines / 72 KB (docs check warns).
- Fix: move lines 15-end (Godot, cloud and transition runs) and the two stale paragraphs unchanged to
  `docs/archive/AGENT_MAP_2026-10.md` with an archive line and a README row; Active now keeps one current paragraph.
  Then either log M0/M1 runs or retire the finished-run log on desktop (Q-F).
- Touches: `docs/AGENT_MAP.md`, `docs/archive/AGENT_MAP_2026-10.md` (new), `docs/archive/README.md`.

### CLN-26

Workflow docs. Part A and B. Status: proposed. Confidence: high.

- `docs/workflow/desktop.md:15` "Create worktrees ... from the latest `develop` ... `git worktree add ../ngp-<id>`"
  contradicts `:23` (batched gate: new task worktrees branch from `staging`) and the real names
  (`C:/dev/tactical-rpg-<id>`, `:28` already uses them); `:44` repeats `../ngp-<id>`.
- `docs/workflow/README.md:19-21` desktop column: `../ngp-<id>`, branches from the latest `develop`, tasks squash onto
  `develop`; since 2026-10-28 they land on `staging` and reach `develop` by `flush.sh`. `:49` `../ngp-<id>`.
- `docs/WORKFLOW.md` (linked from CLAUDE.md as "tools and how to iterate"): the iteration loop (`:43-52`) still says
  "builder agent, feature branch `feat/<id>-<slug>`, merge to `main`"; the skills list (`:28`) lacks milestone-status,
  task-landing and game-design-library; the 2026-10-15 balance baseline (Godot numbers); the agent run log (`:54-99`)
  stopped on 2026-10-16, its table is broken up by the "Parallel safety" paragraphs, and AGENT_MAP replaced it.
- Fix: correct the desktop lines; move WORKFLOW.md's run log, "Parallel safety" notes and the balance baseline
  unchanged to `docs/archive/WORKFLOW_2026-10.md`; the loop points at `docs/workflow/desktop.md` and the task-landing
  skill; refresh the skills list.
- Touches: `docs/workflow/desktop.md`, `docs/workflow/README.md`, `docs/WORKFLOW.md`, `docs/archive/WORKFLOW_2026-10.md`
  (new), `docs/archive/README.md`.

### CLN-27

GAME_BRIEF pointers. Part A. Status: proposed. Confidence: high (pointers), medium (split shape).

- `games/emberwake/docs/GAME_BRIEF.md:12` names `emberwake_world/src/data.rs`, `source.rs` / `store.rs`, `battle.rs`:
  `emberwake_world/src` holds only `lib.rs` since KIT1; the files are in `kits/rpg/crates/rpgkit_world/src/` (and
  `rpgkit_content`).
- About 30 "where to look" lines send the reader to Godot files frozen in `legacy/`: `:89` "Change damage:
  `damage_calculator.gd` only" (an instruction to edit a frozen file), `:67`, `:68`, `:81`, `:90`, `:92`, `:96` (nine
  `combat_*` UI scripts), `:100`, `:145`, `:149`, `:166`, `:172`, `:180`, `:192`, `:208`. Line 5's Godot-to-engine name
  map does not cover file paths.
- `:76` says portrait cues are "rows in `data/catalog/portrait_cues.json`": the kind exists, the data file does not yet
  (PORTRAITFX1-A approved, not built). `:202` and `docs/STRUCTURE.md:59`: overrides "in
  `games/emberwake/data/assets/manifest.json` via `AssetCatalog.upsert`": the file exists only in `legacy/`.
- GAME_BRIEF is 62.8 KB against the 25 KB limit (housekeeper focus priority 2: split where it helps).
- Fix: each line names the engine or kit module first (the Godot file in brackets only where parity matters); drop
  Godot-only instructions; mark portrait cues "(data with PORTRAITFX1-A)"; split the "where to look" section by domain
  behind an index (battle, world, content, UI, tools).
- Touches: `games/emberwake/docs/GAME_BRIEF.md` (+ new per-domain files), `docs/STRUCTURE.md`.

### CLN-28

MILESTONES coverage. Part A. Status: proposed. Confidence: high (facts); placement is Q-B.

- `games/emberwake/docs/MILESTONES.md`: open rows from 2026-10-26/27 match no milestone, so milestone-status counts them
  outside the slice: LOCALGEN3 (in progress), FOUNDERPORT1-P, PIXEL4, ALINE1-F, MOB1, ARTPROMPT1, EXPLOREVIEW1 (and the
  done LOCALGEN2, PIXEL2, PIXEL3, FOUNDERPORT1, FOUNDERPORT1-R, WAR2H1, FLAKE1).
- The design parents CAMPAIGN1, HOOKS1, CAST1, ROSTER1, STORYUI1, RULESET1, FX1, PORTRAITFX1 do not match their
  `X-*` patterns (KIT1, WINDUP1, STATUSFX1 are listed exactly): add the exact ids or let `progress.py` treat `X-*` as
  matching `X`.
- About 45 pre-slice queued rows (UIR-4, UIR-7, QUESTTRACK1-U, RCLOSE1-U, HEAL1-U, BREPORT1-U, MORALE1-U, OBJ1-U,
  PACE1-A/S/U/B, LEVERS1-*, KEEP1, DIFF1, DEEDS1, RESPEC1, OFFHAND1, DESERT1, ASHPACE1, COMBO-A2, MAPNAV1-U, DRAG1,
  PROPS1-A2) show as "ready to start" outside the slice: Q-B.
- Fix: add the ids or patterns the user places (recommend: the art pipeline rows to M2, EXPLOREVIEW1 to M2). Touches:
  `games/emberwake/docs/MILESTONES.md`, optionally `.claude/skills/milestone-status/progress.py`.

### CLN-30

Archive built Godot-era game specs. Part B. Status: proposed. Confidence: medium-high.

- Files (`games/emberwake/docs/specs/`), every task done, no open row cites them, rules live in the engine (PORT_NOTES)
  and the decision files: ART1, AUTO1, BAL2_A, BAL2_L, BAL2_PROPOSAL, BAL3, BOSS1, COMBAT_UI2, DISPLAY1, DLG1_A, DLG1_L,
  EXP1_A, EXP1_L, FORGE1_CLASSES (+_NOTES), FORGE1_ITEMS (+_MATRIX, +_NOTES), INTENT1, ITEMUSE1, M4_A, M4_L, MAPREAD1,
  REP1_A, REP1_L, REVIEW_F, S9_RULES, S10_RULES, S11_RULES, S12_RULES, S12C_A/L, S12S_A/L, S12T_A/L, S14_RULES,
  S14_A1/A2, S14_L1/L2/L3, SPRITE1, TRAV1_A/L, TRAV2_A/L, UI1_A/L, UI2_L, UI3_L, UI_MENUS_RULES (52 files).
  Inbound links outside decisions: ROADMAP's done tables, `games/emberwake/data/{dialogue,maps,quests}/README.md`,
  `games/emberwake/assets/README.md`, sibling specs.
- Keep in place (cited by an open row, a test or a live doc): BAL2_SIM (sim tests), PHASEFOLD
  (`tests/phasefold.rs`), SKILLUI1 (`tests/skill_ranks.rs`), MUSIC1, PLAYTEST1_L (PLAYBACK2, PLAYTEST2), RECRUIT1
  (DESERT1's input), CAMP1, BREPORT1, HEAL1, MORALE1, OBJ1, PACE1, LEVERS1, BAL5_LEVERS, UI_REVIEW1, DESIGN_REVIEW1-3,
  REWARD1, PLAYUI1, DRAG1, ART2*, ART3_PROMPTS, ART_PROMPT_TEMPLATES, PROPS1*, FORGE_SHARED (live rules), every M0-M2
  spec.
- Fix: move to `games/emberwake/docs/archive/specs/` (the game layer's archive, SPLIT1-9) with the archive line and a
  README row; re-point the live links listed above; decision rows stay unchanged (append-only; the archive README maps
  old paths).
- Risk: an agent following an old decision link lands on a moved file (the README resolves it). Touches: the files
  above, `games/emberwake/docs/archive/` (new), `games/emberwake/docs/ROADMAP.md`, the three data READMEs,
  `games/emberwake/assets/README.md`.

### CLN-31

PURSE1 superseded. Part B. Status: proposed. Confidence: high.

- `games/emberwake/docs/specs/PURSE1.md` reads `approved`; REWARD1 replaced it (PURSE1-A/-B rows `superseded by
  REWARD1`, TASKS.md:49-50; HISTORY 2026-10-21 rows "Battle rewards on a win only" and "PURSE1 `purse.loot`"). PURSE1-L
  was built on Godot.
- Fix: archive with CLN-30 (`Archived 2026-10-29: superseded by REWARD1`); re-point `docs/handover/assets-audio.md`
  (or archive it, CLN-32). Touches: `PURSE1.md`, the archive README.

### CLN-32

Cloud handovers and CLEANUP Pass 1. Part B. Status: proposed. Confidence: medium-high.

- `docs/handover/*.md` (7 files, 219 lines): the 2026-10-18 cloud wind-down state of each area thread (queue
  positions, branches, Godot tasks), last edited 2026-10-18/19 (assets-audio.md 2026-10-23). Decision "Fresh
  area sessions" has a thread replace its file at each restart, so a switch back to the cloud writes new ones; the
  current files would mislead a resumed thread. Inbound: `games/emberwake/docs/preferences/README.md` (design.md),
  `docs/workflow/cloud.md` (platform.md as an example).
- Keep (cloud is the documented fallback, `docs/workflow/README.md`): `docs/workflow/cloud.md`, `web.md`,
  `design-review.md`, `content-edits.md`, `docs/metrics/` (TASK_LOG and usage are cloud-only logs).
- Fix: move the seven files to `docs/archive/handover/` with an archive line ("cloud wind-down 2026-10-18; a resumed
  thread writes a fresh handover, cloud.md"); re-point the preferences README. Move CLEANUP Pass 1 (resolved) to
  `docs/archive/CLEANUP_PASS1.md` so this file stays under 25 KB.
- Touches: `docs/handover/*`, `docs/archive/`, `games/emberwake/docs/preferences/README.md`, `docs/specs/CLEANUP.md`.

### CLN-33

Docs status check in the gate. Part C. Status: proposed. Confidence: high. Build first.

- What: `tools/docs_status.awk` (awk, like `md_lint.awk`, so it runs in CI and in Git Bash without Python), called by
  `docs_check` in `tools/fc.sh` and printed as its own `[status]` block (today 30 size warnings drown new lines). Reads
  `docs/TASKS.md`, `docs/archive/TASKS_DONE.md`, `games/emberwake/docs/ROADMAP.md`, `MILESTONES.md` and the first 25 lines
  of each spec.
- Checks: (S1, FAIL) an id in both TASKS and TASKS_DONE, or twice in TASKS (would have caught BAL5-R1b); (S2, WARN)
  `done`/`superseded`/`closed` rows left in TASKS, count plus the oldest date; (S3, WARN) a ROADMAP system row
  `proposed`/`approved` while one of its `X-*` tasks is done, or not `done` while all are done (CLN-22); (S4, WARN) a
  spec named after a system (`X.md`, `X_TASKS.md`) whose newest Status line says `proposed`/`approved`/`spec` while all
  `X-*` tasks are done, or `proposed` while one is done (CLN-23); (S5, WARN) an open row matching no MILESTONES
  pattern (CLN-28); (S6, WARN) backticked repo paths in GAME_BRIEF, STRUCTURE, engine/README, kits/rpg/README,
  CLAUDE.md and `docs/workflow/*.md` that do not exist (suffix match on `git ls-files`, skipping `legacy/` paths and
  lines tagged "Godot") and open-row Touches under a façade crate's `src/` other than `lib.rs` (CLN-21, CLN-27).
- Size S-M; lane logic-builder; Layer engine (repo tools); Depends: CLN-19, CLN-22, CLN-23 first, or the first run
  reports their findings again. Risk: none to the build (warnings; S1 fails only on a real duplicate). Touches:
  `tools/docs_status.awk` (new), `tools/fc.sh`, `docs/STRUCTURE.md` (tools row), `docs/WORKFLOW.md`.

### CLN-34

Landing-time reminders. Part C. Status: proposed. Confidence: high.

- `.claude/skills/task-landing/docs_needed.py`: (a) when the diff marks a task done and every `X-*` row of its system is
  then done, want `games/emberwake/docs/ROADMAP.md` and the system's spec Status line; (b) when a task's new status text
  holds "as built", "user-confirmed", "instead", "pending the user" or "deviat", want the spec (an As-built line) and,
  for "pending", a decision row or an open question; (c) a done row still in TASKS: "move it at the next flush".
- `task-landing` SKILL.md: section 3 gains a flush step "move done rows to TASKS_DONE, refresh AGENT_MAP Active, run
  `fc.sh docs` and read the `[status]` block"; reconcile with planner skill step 6 (it says move, task-landing says
  set `done` in TASKS).
- Lane planner (`.claude/*`). Touches: `.claude/skills/task-landing/{SKILL.md,docs_needed.py}`, `.claude/skills/planner/SKILL.md`.

### CLN-35

Worktree vs status check. Part C. Status: proposed. Confidence: medium.

- A desktop-only helper (beside `stage.sh` in the planner dir, or a `--worktrees` flag on `progress.py`): for each
  `git worktree list` entry on a `claude/<id>` branch, warn when the row is not `in-progress`, or is `done` and the
  worktree still exists (CLN-20); for each `in-progress` row, warn when no worktree or branch names it (LOCALGEN3, ART2).
  Run at dispatch and at wind-down.
- Lane planner. Touches: the planner's helper or `.claude/skills/milestone-status/progress.py`.

### Pass 2 questions

Confirmed (user 2026-10-29, all as recommended; decision "Docs audit pass 2 answers" in `docs/decisions/workflow.md`):
Q-A G1-G6 fold into MILESTONES as a post-slice Ashlands milestone; Q-B a "Later" backlog milestone; Q-C CLN-9 is done;
Q-D regenerate the catalog from the kit; Q-E close LOCALGEN3, ART2 waits; Q-F AGENT_MAP keeps only "Active now".

- Q-A: ROADMAP goals G1-G6 (2026-10-17, Ashlands loop) still read in-progress/approved while the slice milestones
  M0-M2b (MILESTONES.md, decision "Slice group") order the work. Mark G1-G6 as the Ashlands track after the slice, or
  supersede them by the milestones (HISTORY row)? Not re-litigating: only the label is open.
- Q-B: about 45 rows approved 2026-10-17..22 for the Ashlands (CLN-28 list) sit in TASKS as queued and "ready". Keep
  them queued under a "Backlog (after the slice)" milestone row, or set a `parked` status? Dropping any needs the user.
- Q-C: CLN-9 is `done` in TASKS_DONE but "obsolete since the transition" in ROADMAP's refactor backlog; which holds?
- Q-D: `docs/RPG_KIT_CATALOG.md` (2026-10-21 review copy written by `tools/worldbook.py`, Ashlands counts) predates the
  kit (KIT1). Regenerate it from the kit, or retire the output? `WORLDBUILDING.md` stays (Worldbook feedback pending).
- Q-E: LOCALGEN3 has been `in-progress` since 2026-10-26 with no branch (its Klein tests fed FOUNDERPORT1-R and
  STAGE1-A2), ART2 since 2026-10-21 "waits for alpha feedback". Close LOCALGEN3 with what was tested, and give ART2 a
  waiting status, so neither shows as running?
- Q-F: keep logging finished runs in AGENT_MAP on desktop (resume at M0), or retire that log (TASKS_DONE, git log and
  milestone-status cover it) and keep only "Active now"?

### Pass 2 checked and kept

`docs/workflow/cloud.md`, `web.md`, `design-review.md`, `content-edits.md` (cloud is the documented fallback);
`docs/metrics/` (cloud-only logs); `games/emberwake/docs/WORLDBUILDING.md` (feedback pending); `docs/ROADMAP.md`
(the one-line pointer, decision SPLIT1-9 answers); the TASKS_DONE double rows of BAL3-*, BAL4-ATTR, BOSS1-B, CAMP1-B,
OBJ1-B, RECRUIT1-B (a release-log line plus the full row; append-only, harmless once CLN-33's S1 reads TASKS_DONE ids
as a set); engine/README crate rows (only `aegis_app` is gone, labelled retired); DESIGN_REVIEW1-3 (open rows cite
them); `tactical-rpg-dev` (run_dev's worktree).

## Pass 1 (2026-10-17, main at db1ec88)

Resolved; archived unchanged on 2026-10-29 (CLN-32) in [../archive/CLEANUP_PASS1.md](../archive/CLEANUP_PASS1.md).
