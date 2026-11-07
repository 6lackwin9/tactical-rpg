# Tactical RPG (Aegis engine, Rust)

The game runs on Aegis, its own Rust engine in `engine/` (since 2026-10-19). The Godot 4.7 build it replaced is frozen in
`legacy/` (`legacy/LEGACY.md`): a frozen archive for reference, never edited; no build, test or tool reads it, and the
`legacy` branch resumes Godot work only if the user asks.

Layers (SPLIT1, `docs/specs/SPLIT1.md`, done 2026-10-20): the engine (`engine/`, crates `aegis_*`) is game-agnostic; the
RPG kit is `kits/rpg/` (its genre rules in crates `rpgkit_*`, including the battle command set `rpgkit_host`, and their
field tables `schema.json`, since KIT1-M/N; the kit names no game; plus kit assets); the game is `games/emberwake/`
(crates `emberwake_*`, façades over the kit plus game code, `data/`, `assets/`, `tuning.json`, its manifest
`game.json` and its design docs in `docs/`). Every task row has a Layer (engine, kit, game) and its change lands in that
layer; the engine never names a game.

Environment: `$CLAUDE_CODE_REMOTE` empty = **desktop** (read `docs/workflow/desktop.md`), `true` = **cloud** (read `docs/workflow/cloud.md`). Both share everything in this file; what differs (sessions, branches, commits, merges, gate time, IDE rule) and how to switch: `docs/workflow/README.md`.

## Two-session split: Design vs Build (since 2026-11-07)

Design and implementation run in two separate sessions communicating through single-writer handoff docs:

- **Design Session (owns design & shared docs)**: Sole writer of `docs/READY.md`, `docs/specs/MASTERPLAN1.md`, `docs/sessions/*.md`, `docs/specs/*`, `docs/DECISIONS.md` (`docs/decisions/*`, `games/emberwake/docs/decisions/*`), `docs/TASKS.md`, `games/emberwake/docs/ROADMAP.md`, `games/emberwake/docs/MILESTONES.md`, `docs/HISTORY.md`, `README.md`, `games/emberwake/docs/GAME_BRIEF.md`, `docs/STRUCTURE.md`, and `CLAUDE.md`. Runs `tools/fc.sh docs` before every commit. Never edits `docs/FEEDBACK.md` or `docs/archive/FEEDBACK_CLOSED.md` (writes responses in `docs/READY.md` section 4).
- **Build Session (owns code, data, tests & sims)**: Reads **only** `docs/READY.md` (and the specific spec files listed there), `docs/STRUCTURE.md`, and `engine/README.md`. Builds only items marked `ready-to-build` in `docs/READY.md` and runs read-only code audits on items marked `feasibility-check`. **Never edits specs or shared docs**; writes build status, feasibility findings, spec questions, sim results, and docs deltas (`README`, `GAME_BRIEF`, `STRUCTURE`, `HISTORY` lines) **exclusively to `docs/FEEDBACK.md`** (and archives answered/closed items into `docs/archive/FEEDBACK_CLOSED.md` to stay under 25 KB, plus crate-local READMEs where appropriate).

Read these by role instead of crawling the code or historical specs (`docs/specs/README.md` indexes active vs reference specs):

- `docs/READY.md`: **Build Session entry point** (ordered `ready-to-build` and `feasibility-check` queue + Design's answers to feedback)
- `docs/FEEDBACK.md`: **Build Session output log** (build status, feasibility notes, spec questions, docs deltas)
- `docs/specs/MASTERPLAN1.md` and `docs/sessions/*.md`: canonical 9-block / 25-sub-block bottom-up master plan and `.1` design session specs
- `docs/specs/README.md`: separates Tier 1 active specs from Tier 2 pre-masterplan reference library (pull Tier 2 only during `.1` design)
- `docs/STRUCTURE.md`: where code goes (engine and game crates, data, tools, docs by layer)
- `games/emberwake/docs/GAME_BRIEF.md`: layers, core rules/numbers, where to look for X
- `docs/DECISIONS.md`: index of settled decisions; rows in `games/emberwake/docs/decisions/<system>.md` and `docs/decisions/workflow.md`
- `games/emberwake/docs/preferences/`: Tushar's design, UI and asset preferences
- `docs/HISTORY.md`: deprecated/tried ideas with reasons (append-only; Design applies rows from Build's docs delta)
- `games/emberwake/docs/BALANCE_REPORT.md`: latest balance runs and open issues
- `docs/TASKS.md`: compact reference map of open rows (mapped to masterplan tasks in `docs/specs/MASTERPLAN1_ROWS.md`; full historical notes in `docs/archive/TASKS_PRE_MASTERPLAN.md`)
- `docs/AGENT_MAP.md`, `docs/WORKFLOW.md`, `docs/EFFICIENCY.md`, `engine/README.md`, `engine/docs/PORT_NOTES.md`
- Skills `engine-checklist`, `known-issues`, `rust-conventions`

## Roles and lanes (inside the Build Session)

- **Build Planner** (skills `planner`, `planner-ops`): reads `docs/READY.md`, dispatches lanes (`logic-builder`, `asset-builder`, `balance`, `change-reviewer`), merges to `staging`/`develop`, and records status, feasibility notes, and docs deltas in `docs/FEEDBACK.md` (does not edit `TASKS.md`, `ROADMAP.md`, decisions, `HISTORY.md`, or shared READMEs directly).
- `design-brainstorm` agent (Design Session only): options and specs in `docs/sessions/` or `docs/specs/`.
- `logic-builder` agent (logic lane): code in the task's layer (`engine/crates/**` engine, `kits/rpg/crates/**` kit, `games/emberwake/crates/**` game), schema, validation, tests and tools on the task's branch (desktop: a feature worktree); never edits shipped data; never commits.
- `asset-builder` agent (asset lane): fills `games/emberwake/data/` through `aegis content` (upsert / remove / apply a seed) against a merged schema; no code, no tests.
- `balance` agent (balance lane): sims on `balance-ready` systems; reports findings for `docs/FEEDBACK.md` / `BALANCE_REPORT.md`.
- `change-reviewer` agent: read-only pre-commit review.
- `housekeeper` agent (cleanup lane): finds legacy/redundant code, dead paths and bloat; proposes trims in `docs/specs/CLEANUP.md`.

## Standing rules

Strict static typing (Rust; the build has no compiler warnings, clippy is advisory); builders report a docs delta in `docs/FEEDBACK.md` and the Design Session applies it to shared docs; push to `develop` or `main` only when `check_project.ps1 -Fast -Quiet` (desktop) or `tools/fc.sh gate` (cloud; same steps: build every crate, run every test, including `tools/planner/` tests) is clean and `tools/fc.sh docs` is clean (desktop: a task may first land on the local `staging` branch after a partial gate, and `staging` reaches `develop` in batches under one full gate; decision "Batched gate"); on desktop also the IDE Problems tab has 0 errors/warnings (markdownlint compliant; see `.agents/rules/engine_workflow.md` Section 6); `main` is always a clean build (features on branches); edit content only through `aegis content` (`tools/fc.sh content`), never raw JSON; never edit `legacy/`; no sed line-range edits; docs history is append-only (mark deprecated with reason, never delete).

Engine goldens (the Godot parity suites were retired 2026-11-01, CLN-36..42; `legacy/` is not a test oracle): `aegis_core` rng_hash, `emberwake_rules` rules_golden, `emberwake_sim` sim_golden, `emberwake_content` checks_golden, `rpgkit_host` rules_contract_golden (`BASERULES1-M`), the `emberwake_world` journey, the store byte-for-byte resave, the timeline golden and the PHASEFOLD golden. A golden is re-recorded only for an intended change, named in the commit and a `docs/HISTORY.md` row; never edit a golden to make a test pass; an unintended divergence is a bug (fix the engine).

## Feedback rule

Look up the failure in the known-issues table first; after two failed targeted fixes, stop and ask the user (state findings + options). Always ask before design/gameplay decisions. Add new recurring failure causes to the known-issues table.
