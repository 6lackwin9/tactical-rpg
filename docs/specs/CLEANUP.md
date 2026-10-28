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
| 1 | CLN-19 | A | Move 97 done and 2 superseded rows out of TASKS; drop the stale BAL5-R1b duplicate | S | planner | at the next flush | high | confirmed |
| 2 | CLN-20 | A | TASKS statuses vs live work: 3 running tasks read `approved`, 2 spec rows unfinished, 1 stale worktree | S | planner | now | high | confirmed |
| 3 | CLN-22 | A | ROADMAP system rows: 13 statuses behind their tasks, answered questions still "open", FX1 missing | S | planner | now | high | confirmed |
| 4 | CLN-33 | C | `tools/docs_status.awk` in `fc.sh docs`: status consistency checks (build first) | S-M | logic-builder | after CLN-19, CLN-22 | high | confirmed |
| 5 | CLN-27 | A | GAME_BRIEF: Godot-file and pre-KIT1 "where to look" pointers; split | M | design-brainstorm | now | high | confirmed |
| 6 | CLN-23 | A | Spec Status lines: 21 `proposed` specs approved since 2026-10-25, 40 built specs still `approved` | S-M | design-brainstorm | now (or with CLN-30) | high | confirmed |
| 7 | CLN-26 | A+B | Workflow docs: desktop worktree and branch steps, WORKFLOW.md loop and run log | S | planner | now | high | confirmed |
| 8 | CLN-25 | A+B | AGENT_MAP: finished-run log stops at SPLIT1; archive the cloud-era log | S | planner | now | high (facts), question (log) | confirmed |
| 9 | CLN-24 | A | Missing As-built notes (HOOKS1, CAMPAIGN1-C) and an open user pick buried in a done row | S | planner | now | medium | confirmed |
| 10 | CLN-21 | A | Open task rows' Touches name files that moved to `kits/rpg` in KIT1 | S | planner | now | high | confirmed |
| 11 | CLN-28 | A | MILESTONES: recent art tasks and ~45 pre-slice queued rows match no milestone | S | planner + user | now | high (facts), question (placement) | confirmed |
| 12 | CLN-34 | C | `docs_needed.py` and task-landing: ROADMAP, spec status, As-built and done-row reminders | S | planner | with CLN-33 | high | confirmed |
| 13 | CLN-30 | B | Archive 52 built Godot-era game specs | M | design-brainstorm | after CLN-23 | medium-high | confirmed |
| 14 | CLN-31 | B | Archive PURSE1.md (superseded by REWARD1) | S | design-brainstorm | with CLN-30 | high | confirmed |
| 15 | CLN-32 | B | Archive the cloud handover files and CLEANUP Pass 1 | S | planner | now | medium-high | confirmed |
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

Resolved (checked 2026-10-29 in Pass 2): every candidate is done or closed; rows in `docs/archive/TASKS_DONE.md`.

Scope: SKILL1 legacy, superseded ideas, dead code, seeds, test helpers, oversized docs and suites (read-only `git grep` scan). Overlap checked against the Touches of BAL2-L, BAL2-A, ITEM1 and CLASS1 in `docs/TASKS.md`.

### Ranked summary

| Rank | Id | What | Size | Lane | When | Status |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | CLN-1 | Fix doc drift that misleads agents (skill cap, ArenaTestSuite, stale TASKS rows, HISTORY layout) | S | planner (docs) | now | done (docs/) 2026-10-17 |
| 2 | CLN-2 | Archive the 9 executed seeds in `data/seeds/` | S | asset-builder | now | done (cbd475c) |
| 3 | CLN-3 | Remove SKILL1 legacy normalisation and the `offhand` / `advanced` shims | S-M | logic-builder | after BAL2-L, after CLN-2 | done (CLN-3b 2026-10-18) |
| 4 | CLN-4 | Archive `.context/` (10-block design docs) | S | planner (docs) | now | done (docs/) 2026-10-17 |
| 5 | CLN-5 | Archive done specs; slim FORGE1_SKILLS and MATRIX to a shared Forge rules file | M | planner / design | before ITEM1 and CLASS1 dispatch | done 2026-10-17 |
| 6 | CLN-6 | Delete dead functions and legacy class constructors | S | logic-builder | after BAL2-L (ItemCatalog part after ITEM1) | done (TASKS_DONE) |
| 7 | CLN-7 | Drop the second unit-power formula in `CharacterStats` | S | logic-builder | after BAL2-L | done (TASKS_DONE) |
| 8 | CLN-8 | Move repeated test helpers into `TestHelpers` | M | logic-builder | after BAL2-L, before ITEM1 | done (TASKS_DONE) |
| 9 | CLN-9 | Split `content_catalog.gd` (1384 lines) by kind | L | logic-builder | after BAL2-L, before ITEM1 and CLASS1 | done (TASKS_DONE; see Pass 2 Q-C) |
| 10 | CLN-10 | Split DECISIONS.md and move superseded rows (parked item) | M | planner (docs) | user choice | done (docs/) 2026-10-17 |
| 11 | CLN-11 | Trim ROADMAP.md of decided brainstorm options and done plans | S-M | planner (docs) | now | done 2026-10-17 |
| 12 | CLN-12 | Split suites over the 450-line rule | M | logic-builder | after ITEM1 | done (TASKS_DONE) |
| 13 | CLN-13 | README overview plus per-topic files (parked item) | L | planner (docs) | user choice | done 2026-10-19 (RESCOPE1) |
| 14 | CLN-14 | Remove the `ArenaTestSuite` pass-through | S | logic-builder | with CLN-8 | done (TASKS_DONE) |
| 15 | CLN-15 | Map the existing goblin placeholder art to units (new, from Q1) | S | asset-builder | now, not in parallel with BAL2-A | done (TASKS_DONE) |
| 16 | CLN-16 | Remove the legacy `smart_skills = false` AI path (new, from Q2) | S | logic-builder | after the BAL2 rerun | done 2026-10-17 |

### Confirmation and sequencing (user 2026-10-17, DECISIONS "Cleanup pass 1 answers")

All candidates confirmed; TASKS rows `CLN-<n>` carry status `approved`. Lanes: docs-only work goes to design-brainstorm (the planner keeps README, CLAUDE.md and `.claude/*`), seeds and data to asset-builder, code to logic-builder.

- Now (docs, archive, seeds; no builder-file overlap): CLN-1, CLN-2, CLN-4, CLN-5, CLN-11, CLN-10 (after CLN-1), CLN-15 (before BAL2-A starts or after it merges: both edit `units.json`).
- After BAL2-L merges, before ITEM1: one logic cleanup branch in this order: CLN-14, CLN-8, CLN-6, CLN-7, CLN-3, CLN-9 (pure move last, after the content edits).
- After ITEM1: CLN-12 plus the ItemCatalog part of CLN-6. After one release with no deprecation warnings: CLN-3b. After the BAL2 rerun: CLN-16.
- Parked: CLN-13.
- Answers folded in: CLN-10 splits by system; CLN-4 gets a short integration pass before the drop; CLN-3 deprecates v1 loading and the old-key normaliser first (CLN-3b drops them). Q1 became CLN-15, Q2 became CLN-16; Q3 stays a review after ITEM1.

### CLN-1

Doc drift. Status: done for docs/ 2026-10-17.

- What and where: agent-facing facts that the code no longer has. `games/emberwake/docs/GAME_BRIEF.md:22` "skill cap = ceil(level/2)" and `:57` "within the unit's skill cap" (SK2 removed the cap, DECISIONS rows SK2 picks / SK2-L built); `:52` riposte "L2, learned_only, guard 0.15, 2 rounds" (now skill data, DECISIONS "M2C2B content"); `:93` "Advanced families (`advanced`)" (now `grade`, HISTORY 2026-10-16 row); `docs/STRUCTURE.md:44` and `.claude/agents/logic-builder.md:22` register suites in `ArenaTestSuite.run_all_tests` (now `SuiteRegistry`, DECISIONS "R1 test split"); `.claude/skills/godot-debug-checklist/SKILL.md:42` and `.claude/skills/patch-and-seed/SKILL.md:17` cite the skill cap; `README.md:79,121,294,596` skill cap, `README.md:381` "20 test suites, 171 assertions" (34 suites now). `docs/TASKS.md:8-9` keep C1 and D1 `in-progress` although their asset parts were folded into M2C2B (done) and DECISIONS records the C1-L, C1-A1 and D1-L reviews; `docs/TASKS.md:17` still lists "open question: `evolves_from`" (resolved, DECISIONS "FORGE1 skill answers 2" and "round 4"). `docs/HISTORY.md:19-22` the Lessons section sits inside the table, so rows 23-28 render outside it.
- Change: correct the lines; set C1, D1, FORGE1 rows to `done`; move the Lessons block below the table (no row text changes, append-only kept). Leave `GAME_BRIEF.md:65` (UNARMED_VALUE 14) to BAL2-A, which changes it.
- Risk: none in code. `.claude/*` edits are the planner's with the user's consent.
- Touches: `games/emberwake/docs/GAME_BRIEF.md`, `docs/STRUCTURE.md`, `docs/TASKS.md`, `docs/HISTORY.md`, `README.md`, `.claude/agents/logic-builder.md`, `.claude/skills/godot-debug-checklist/SKILL.md`, `.claude/skills/patch-and-seed/SKILL.md`.

### CLN-2

Archive executed seeds. Status: confirmed (user 2026-10-17).

- What and where: `data/seeds/c1a1.gd`, `m2c2b_a.gd`, `m2c2b_b.gd`, `m2c2b_c.gd`, `m2c2b_d.gd`, `m2c2b_lib.gd`, `sk2.gd`, `sk2_schema.gd`, `skill1_ranks.gd` (803 lines). Their tasks are done (TASKS rows C1, M2C2B, SK2, SKILL1-A); nothing references them except the `m2c2b_*` preloads of `m2c2b_lib.gd`.
- Why bloat: (1) they encode superseded data: `m2c2b_lib.gd:44` writes `"offhand": false`, `sk2.gd:28` writes `advanced`, `m2c2b_a.gd` re-creates pre-SK2 paths and mage `volley`, so a re-run would revert shipped data; they are the only reason the legacy normaliser of CLN-3 must stay. (2) `AssetScan.SCAN_DIRS` (`scripts/content/asset_scan.gd:11`) includes `res://data/seeds`, so every id they quote counts as a reference: `ContentCatalog.demote` refuses with "named by tests or seeds" (`content_catalog.gd:458-460`) and SAFE DELETE refuses too, for almost every shipped skill and class. (3) `parse-all` compiles them on every run. DECISIONS "C1-A1 assets" keeps seeds idempotent for re-runs after a rebase, which no longer applies once merged.
- Change: remove them from `res://` (git history keeps them) and add `data/seeds/README.md` with one line per seed (task, merge commit: `11ffe67`, `b7c4016`, `9686a36`, `7bed659`, `745962a`) plus the rule "a seed leaves this folder in the cleanup pass after its task is merged".- Risk: low. `origin_test_suite.gd:158-188` uses its own fixtures, not seed hits. BAL2-A's coming `data/seeds/bal2.gd` is unaffected. Update `docs/STRUCTURE.md:17,29`.
- Touches: `data/seeds/*` (new README), `docs/STRUCTURE.md`.

### CLN-3

SKILL1 legacy shims. Status: confirmed (user 2026-10-17); CLN-3 done, CLN-3b done 2026-10-18 (no warning hit in the gate or shipped data).

- What and where: `ContentSchema._legacy_skill` (`scripts/content/content_schema.gd:369,379-392`, comment "accepted for one release"); derived properties `SkillDef.offhand` (`scripts/combat/planning/skill_def.gd:72-77`) and `SkillDef.advanced` (`:101-106`); their readers `tactical_ai.gd:168,171`, `plan_entry.gd:62`, `skill_catalog.gd:90`, `damage_calculator.gd:196`, `dojo_session.gd:458-459,513,618,621,630`, `forge_preview.gd:224`, `combatant.gd:284,299`, `class_rules_test_suite.gd:175`; tests of the removed behaviour `skill_model_test_suite.gd:147-151,160,254`; `content_test_suite.gd:115` still feeds the old `"advanced": true` key.
- Why bloat: HISTORY 2026-10-16 rows (bool `advanced` / `evolves_from`, off-hand categories) and DECISIONS "Strict hand typing" / "round 4". Shipped data is clean: `skills.json` is version 2 with no `advanced`, `offhand` or `evolves_from` keys (the 11 hits are `"grade": "advanced"`), no world file has them, `evolves_from` is already gone from code.
- Confirmed shape (user: deprecate first, drop once unused): in CLN-3 `_legacy_skill` and v1 skills loading warn on use; CLN-3b deletes both when logs show no hit. Shims and legacy-key tests go in CLN-3.
- Change: delete `_legacy_skill` (CLN-3b); readers use `weapon_hand == SkillDef.WeaponHand.OFF` (one helper such as `is_off_hand()` is fine) and `grade != SkillDef.Grade.NOVICE`; delete the legacy tests, rewrite line 115 with `grade`. Optional: drop version 1 skills-file reading (`content_catalog.gd:143` keeps v1 on the generic path) and its check `skill_ranks_test_suite.gd:196-203`; small gain, ask. Keep `WeaponHand.BOTH` and `BOTH_ENABLED` (DECISIONS "Strict hand typing": `both` is an engine follow-up).
- Risk: an old JSON with the bool keys would silently lose them; none exists after CLN-2. Covered by skill_model, content, class_rules, skill_family, ai_skills, dojo suites. Assertion count drops by the removed legacy checks only.
- When: after BAL2-L (shares `content_schema.gd`) and CLN-2; `combatant.gd` is in ITEM1's Touches, so run before ITEM1 starts or after it merges.
- Touches: `content_schema.gd`, `skill_def.gd`, `tactical_ai.gd`, `plan_entry.gd`, `skill_catalog.gd`, `damage_calculator.gd`, `dojo_session.gd`, `forge_preview.gd`, `combatant.gd`, suites skill_model, content, class_rules (optionally content_catalog, skill_ranks).

### CLN-4

Archive `.context/`. Status: done for docs/ 2026-10-17; the planner deletes the folder.

- What and where: `.context/COMBAT_SYSTEM.md` (14 KB, "Fixed 10-Block Timeline", six stances), `.context/GAME_DESIGN.md` (10 KB, "10-Block Tactical Combat"), `.context/AGENTS.md` (4 KB coding standards); last edited 2026-10-10/11.
- Why bloat: HISTORY row 1 deprecated the 10-block scripted layer; the coding rules repeat `CLAUDE.md` and `.agents/rules/godot_workflow.md`. Only `README.md:604,613,621` point at GAME_DESIGN.md (as the spec of the placeholder systems S9-S11).
- Change (confirmed shape): a quick pass for anything still useful (candidates: GAME_DESIGN systems 1, 2 and 9 as input for ROADMAP P-S9..P-S11; any coding rule missing from CLAUDE.md or `.agents/rules`), integrate it into the current docs, then drop the folder (git history keeps it) with a note in `docs/archive/README.md`; repoint the README lines to ROADMAP.
- Risk: none in code. Question: does an IDE integration still read `.context/AGENTS.md`? If yes keep that one file.
- Touches: `.context/*`, `docs/archive/*`, `README.md`.

### CLN-5

Archive done specs. Status: done 2026-10-17.

- What and where: (a) specs of done tasks whose headers still say `proposed`/`approved`: `AI1.md:3`, `B1.md:3`, `C1.md:3`, `D1.md:3`, `M2C2B.md:3`, `M3.md:3`, `R1.md:3`, `SK2.md:3` (about 88 KB). (b) `FORGE1_DESIGN.md` (23 KB): skills part already archived, items superseded by FORGE1_ITEMS (HISTORY 2026-10-16 items row), classes by FORGE1_CLASSES, units section 5 restated by `FORGE1_CLASSES.md` section 8. (c) `FORGE1_SKILLS.md` (40 KB) and `FORGE1_SKILLS_MATRIX.md` (28 KB), both over the 25 KB limit (`tools/fc.sh docs`); SKILL1 is done.
- Change: mark (a) `done` and move to `docs/archive/specs/`; archive (b); for (c) copy the parts the next passes still cite into `games/emberwake/docs/specs/FORGE_SHARED.md` (section 12 shared Forge rules, 11.3 ownership, 4.1 `weapon_hand`, matrix M origin key fields; about 10 KB, keep the section numbers so code comments such as `asset_scan.gd:5` "FORGE1 section 12.5" and `skill_def.gd:31` "FORGE1 11" still resolve) and archive the rest. Repoint `FORGE1_CLASSES.md:4,29,117,234`, `FORGE1_ITEMS.md:96,172`, `TASKS.md:17-19`.
- Leave `FORGE1_ITEMS.md` (40 KB) and `FORGE1_CLASSES.md` (26 KB) alone: ITEM1 and CLASS1 build from them; slim them the same way after those merge.
- Risk: none in code; a builder brief pointing at an old path. Do it before ITEM1 / CLASS1 are dispatched so their briefs cite stable paths.
- Touches: `docs/specs/*`, `docs/archive/*`, `docs/TASKS.md`, `games/emberwake/docs/specs/README.md` (note that done specs move to the archive).

### CLN-6

Dead code. Status: confirmed (user 2026-10-17).

- What and where (zero callers in scripts, scenes, tests, tools; checked with `git grep -w`): `ClassArchetype.create_shaman/create_scout/create_berserker/create_brawler` (`scripts/core/specs/class_archetype.gd:81-92`; three name classes removed by HISTORY 2026-10-15 roster row and would only warn "unknown class"); `CharacterStats.get_ranged_power`, `get_tactical_power` (`character_stats.gd:152-158`, fixed 1.5 scaling, HISTORY "Fixed 1.5 Unit Power" row), `get_intent_unmask_blocks` (`:169-175`), `remove_modifiers_from_source` (`:207`, alias), `tick_modifiers` (`:210-216`), field `experience` (`:19`, M3 uses `total_xp`), header "Masteries" (`:6`, HISTORY Mastery row); private test helpers `engine_pieces_test_suite.gd:209 _backflip_steps`, `ai_skills_test_suite.gd:173 _uses_any`; `world_forge.gd:623 _current_entry`; `ItemCatalog.create_spiked_knuckles`, `create_consumable`, `all_items` (`item_catalog.gd`).
- Verify before deleting (public API that a planned UI may want): `CombatGrid.can_target`, `Combatant.has_target`, `Proficiency.duplicate_set`, `PlanStep.duplicate_step`, `WorldCatalog.entry_count`, `Equipment.get_requirements_display`, `Inventory.get_total_gear_stat_bonus`, `CombatDockHost.is_edit_mode/panel_ids`, `PartyState.member_ids`, `DictUtil.bool_of`, signals `leveled_up` / `attribute_point_allocated` (emitted, never connected).
- Kept on purpose: README roadmap keeps "Reactive intent" (`README.md:558`) as an idea; deleting the unused PERC helper does not drop the idea.
- Risk: low; `parse-all` catches missed callers.
- When: `character_stats.gd` after BAL2-L; `item_catalog.gd` after ITEM1 (the item migration may rename ids).
- Touches: `class_archetype.gd`, `character_stats.gd`, `item_catalog.gd`, `world_forge.gd`, suites engine_pieces, ai_skills.

### CLN-7

Second unit-power formula. Status: confirmed (user 2026-10-17).

- What and where: `CharacterStats.get_melee_power` (`character_stats.gd:148-150`, `weapon + round(STR x 1.5)`) is used only by tests: `character_specs_test_suite.gd:34` and Dojo-10 (`scenes/dojo/dojo_assertions.gd:113`), which compares the combatant's real power with it.
- Why bloat: `DamageCalculator` is the only formula (DECISIONS "Combat extensions", GAME_BRIEF); per-class proficiency scaling replaced the fixed 1.5 (HISTORY). Dojo-10 passes only while the default scaling equals 1.5; BAL2-A "trained fists" (DECISIONS "BAL2 role answers" 5) changes unarmed power and will break this check.
- Change: derive both expectations from `DamageCalculator.attack_power`; delete `get_melee_power`.
- Risk: low; two assertions change their source, count unchanged.
- When: after BAL2-L; tell the BAL2-A brief that Dojo-10 reads a second formula until then.
- Touches: `character_stats.gd`, `character_specs_test_suite.gd`, `scenes/dojo/dojo_assertions.gd`.

### CLN-8

Test helpers into `TestHelpers`. Status: confirmed (user 2026-10-17).

- What and where: 19 private `_report` copies in two shapes (checks dict, or `ok` + detail), e.g. `ai_skills_test_suite.gd:15`, `combat_extensions_test_suite.gd:16`, `engine_pieces_test_suite.gd:12`, `focus_gear_test_suite.gd:16`, `foundation_test_suite.gd:14`, `class_rules_test_suite.gd:14`, `content_test_suite.gd:15`, `core_rules_test_suite.gd:14`; `SkillTestKit.report` (`skill_test_kit.gd:7`) is already the generic one. Five hand-built `_unit` combatant builders with the same body (`ai_skills:114`, `combat_extensions:52`, `riposte:38`, `skill_engine:42`, `weaken:55`). Fifteen suites fork a sandbox with their own check (`ContentCatalog.fork_to(SANDBOX)` vs `SkillTestKit.fork`).
- Why bloat: DECISIONS "R1 test split" names `TestHelpers` as the home of shared helpers; `GAME_BRIEF.md:78` repeats it.
- Change: `TestHelpers.report(tag, label, checks)`, `TestHelpers.report_ok(tag, label, ok, detail)`, `TestHelpers.unit(...)`, `TestHelpers.fork(sandbox)`; fold `SkillTestKit` fork/cleanup/report into it (keep its skill fixtures `attack`, `messages`, `any_contains` or move them too). Print tags stay identical so log greps keep working.
- Risk: low-medium (many files); rule: assertion count before = after.
- When: after BAL2-L (edits class_rules, engine_pieces); before ITEM1 so ITEM1's suites use the shared helpers.
- Touches: `scripts/tests/*`.

### CLN-9

Split `content_catalog.gd`. Status: confirmed (user 2026-10-17).

- What and where: `scripts/content/content_catalog.gd` is 1384 lines: store and IO (1-270), origin (280-462), ranked skills API (464-730), references (733-806), validation hub (808-945), skill rules (946-1137), item rules (1138-1186), class, unit and loadout rules (1190-1384).
- Why: `docs/TASKS.md:21,24` must run CLASS1 and ITEM1 sequentially because of these hub files; ROADMAP "Hub-file overlap" finding 2 deferred an R2 split "if a later item or skill schema change collides", which is now the case.
- Change: pure move of the per-kind rules into `scripts/content/rules/{skill,item,class_unit}_rules.gd` (static, called by `validate_entry` / `warnings_for`); `ContentCatalog` keeps store, IO, origin, references and dispatch. Optional second step: per-kind field tables of `content_schema.gd` (502 lines). No behaviour change.
- Risk: medium (largest file, private cross-calls); fully covered by the content, skill_* and class_rules suites; assertion count equal.
- When: after BAL2-L merges, before ITEM1 / CLASS1 start. Gain: ITEM1 and CLASS1 may then run in parallel if `content_schema` / `content_factory` edits stay small (planner decides).
- Touches: `content_catalog.gd`, new `scripts/content/rules/*`, `docs/STRUCTURE.md`.

### CLN-10

DECISIONS split. Status: done for docs/ 2026-10-17 (user: split by system, moved rows stay reviewable; was parked in EFFICIENCY "Doc housekeeping"; it is 35 KB now, 18 KB when parked, and ITEM1 / CLASS1 / BAL2 will add rows).

- Superseded or duplicate rows (line numbers in `docs/DECISIONS.md`): 12 commit trailer hard-coded (EFFICIENCY "Downtime pass": trailer from the session); 18 gear budget details open (25, 40, 41); 20 support identity pending (27); 24 and 32 build/milestone order (71); 28 duplicates 9; 44 level 4-5 advanced versions (55 says it supersedes); 45 slot counting open (53, 57); 52 tier selection pending (55); 55 "limited by the unit's skill cap" (79, 80); 57 basics as strike and guard families (60, 66); 61 Rest pending (67); 63 riposte AP 1/2/3 (76); 76 interim riposte level 5 (79, 82); 79, 80, 82 bool `advanced` (88); 81 "flags like `advanced`" (88); 85 `evolves_from` (88); 86 dual-wield guard as main rule (87-89); 87 per-hand bands and `offhand` (88); 92 item 11 duel gate (94 item 3).
- Change: `docs/DECISIONS.md` becomes a short index; rows move unchanged into domain files (combat, classes and skills, gear and items, content and Forge, process and balance); superseded rows go to `docs/archive/DECISIONS_SUPERSEDED.md` with a "superseded by" pointer. Add the superseded concepts HISTORY lacks: AP-cost skill unlock and skill cap, class-level active skill loadout (now on the unit), level 4-5 advanced versions, basic-skill tier variants, riposte AP 1/2/3, hard-coded commit trailer.
- Question for the user: is moving a superseded row to the archive (text unchanged, pointer added) compatible with "append-only, never delete"?
- Touches: `docs/DECISIONS.md`, new `docs/decisions/*`, `docs/archive/*`, `docs/HISTORY.md`, `CLAUDE.md` and `.claude/agents/*` pointers.

### CLN-11

ROADMAP trim. Status: done 2026-10-17.

- What and where (`games/emberwake/docs/ROADMAP.md`, 21 KB): "Open questions for the user" (37-42, answered by DECISIONS 25-28); "Proposed designs (brainstorm pass 2026-10-15)" (44-138, options decided by DECISIONS 16-28 and built in M1-M3); "Refactor backlog" R1 (140-144, done); "Milestone plan v2" and its hub matrix (146-178, all done except M4); S2 row note "M2c-2b content next" (14, done); "Open engine items" first two bullets (192-194, AI hooks done in AI1).
- Change: move those sections verbatim to `docs/archive/ROADMAP_2026-10.md`; keep systems table, suggested build order, BAL2 plan, remaining open engine items; set the S2 note to the current state.
- Risk: none. Touches: `games/emberwake/docs/ROADMAP.md`, `docs/archive/*`.

### CLN-12

Large suites. Status: done 2026-10-17.

- Result: suites CombatExtAttacks, CombatExtAi (helpers `CombatExtKit`), AiSkillsTactics (fixtures `AiSkillsKit`), Unarmed, FocusGearBudget; dojo split into DojoSessionAssertions, DojoUiAssertions. Same PASS lines. ItemCatalog: `create_spiked_knuckles`, `all_items` deleted; `create_consumable` kept (in use). Dojo-27 has no caller: CLN-17.
- What: suites over the 450-line rule (DECISIONS "R1 test split"): combat_extensions 795, dojo_assertions 771, ai_skills 634, engine_pieces 480, focus_gear 459.
- Change: split by feature group, one registry line each; no behaviour or number change.
- Touches: those suites, the registry.

### CLN-13

README split. Status: confirmed (user 2026-10-17), still parked until the next large README update (EFFICIENCY item 4).

- What and where: `README.md` 88 KB, 674 lines; "Codebase Architecture & Directory Breakdown" 20 KB repeats STRUCTURE and GAME_BRIEF; "Headless Testing" 8.8 KB of per-milestone notes; "Roadmap & Features Yet to be Implemented" repeats ROADMAP.
- Change: overview README (run, layout pointer, feature list) plus `docs/features/<system>.md`; replace the repeated parts with links. Agents do not read the README, so the value is safer planner edits.
- Touches: `README.md`, new `docs/features/*`.

### CLN-14

`ArenaTestSuite` pass-through. Status: confirmed (user 2026-10-17).

- What and where: `scripts/tests/arena_test_suite.gd` (15 lines) only forwards to `SuiteRegistry.run_all`, and its suite list (8-11) is stale; the only caller is `scenes/test_arena.gd:15`.
- Change: call `SuiteRegistry.run_all` from `test_arena.gd`, delete the file, update `GAME_BRIEF.md:78`, `STRUCTURE.md:13,44`, `logic-builder.md:22`.
- Risk: very low. Touches: `arena_test_suite.gd`, `scenes/test_arena.gd`, docs above.

### CLN-15

Goblin placeholder art. Status: confirmed (user 2026-10-17, new from Q1).

- What and where: `UnitArt.GOBLIN_CLASSES` (`unit_art.gd:17-18`) draws archer, berserker, shaman, skirmisher with goblin skin (plus scout), but every unit has `art: ""`. Unit `art` is a schema enum over `CLASS_KEYS` (`content_schema.gd:251`), read via `ContentFactory.art_of` (`catalog_encounter.gd:141`); world characters inherit it (`world_factory.gd:19-21`): data only.
- Change (asset lane, seed `data/seeds/cln15_art.gd` through `ContentCatalog.upsert`): `goblin_archer` to archer, `goblin_shaman` to shaman, `goblin_skirmisher` to skirmisher, `goblin_berserker` and `ambush_berserker` to berserker, `fleeing_scout` to scout. `goblin_brawler` has no goblin key: berserker (goblin skin) or keep brawler; the builder asks the planner.
- Risk: low; no test pins a unit's art. Not beside BAL2-A (both edit `units.json`).
- Touches: `data/catalog/units.json`, `data/seeds/cln15_art.gd` (archived after merge per CLN-2).

### CLN-16

Legacy AI path. Status: done 2026-10-17.

- What and where: `AiProfile.smart_skills` (`ai_profile.gd`, 2 uses) and its off branches in `tactical_ai.gd` (5 uses); `ai_skills_test_suite.gd` (7 uses) covers both paths; BAL2-L adds a sim switch for it (`tools/sim/*`).
- Change: remove the `false` path, the field, its tests and the sim switch; fix GAME_BRIEF "AI skill use".
- When: after the BAL2 rerun. Risk: low (ai_skills suite).
- Touches: `ai_profile.gd`, `tactical_ai.gd`, `ai_skills_test_suite.gd`, `tools/sim/*`, `games/emberwake/docs/GAME_BRIEF.md` (planner).

### Questions (not proposed for removal)

Q1 and Q2 answered (now CLN-15, CLN-16); Q3 waits for ITEM1.

- Q1 (unused goblin art keys) answered: map them now (CLN-15). Q2 (`smart_skills = false`) answered: remove after the BAL2 rerun (CLN-16).
- Q3 `ItemCatalog` named shorthands (34 uses in `dojo_loadouts.gd`) map old names to item ids (`create_iron_claymore` returns `wpn_claymore`); review after ITEM1's migration. Answered (user 2026-10-17): fold the review into the CLN-6 ItemCatalog part (ITEM1 is done).

### Checked and kept

`WeaponHand.BOTH` / `BOTH_ENABLED` (planned), `StatRules.default_unlock` (live rule), `AiProfile.skill_cap` (SK2; dummy sets 1), slot constants (unit loadout), flat-row `upsert` (live API), unused passive templates (M4 sources), plain dual wield `equipment.gd:80` (superseded by ITEM1).
