# KIT1 tasks: the steps, the path map, task rows and docs delta

Status: proposed (2026-10-25); the design is in [KIT1.md](KIT1.md) (section numbers such as 3.2 refer to it)

## 1. Rules for every step

1. Move only: no rule, number, text or order changes. A clean-up seen on the way goes into the report as a new task.
2. Never edit a file under `games/emberwake/crates/*/tests/`, `games/emberwake/parity/`, `games/emberwake/data/`,
   `legacy/` or any recording or golden file.
3. Gate in a checkout that holds `games/emberwake/parity/out/` (copy it from the main checkout into the worktree, or
   record it with `tools/fc.sh parity`); a suite that skips for a missing recording fails the step.
4. Before the change, save the list of passing tests (`cargo test --workspace --no-fail-fast` summary lines and
   `cargo test --workspace -- --list`); after it, the same names pass. Only a moved module's unit tests may change
   their crate name (`emberwake_battle` to `rpgkit_battle`), nothing else.
5. Save `aegis sim duel 5 2`, `aegis sim campaign 5` and `aegis content check` output before; the same text after.
6. One commit per step on its own branch; `tools/fc.sh gate` (or `check_project.ps1 -Fast -Quiet`) green; the
   builder reports the docs delta (section 6) and never edits docs.

## 2. KIT1-M: the byte-identical move

Schema-first: no (no schema change; the file moves as it is).

- M1. `Cargo.toml` members gain `"kits/*/crates/*"`.
- M2. For each area `x` in `data`, `rules`, `battle`, `content`, `world`, `sim`: every file under
  `games/emberwake/crates/emberwake_x/src/` moves with `git mv` to `kits/rpg/crates/rpgkit_x/src/` (same relative
  path, `check/` included), content unchanged.
- M3. `kits/rpg/crates/rpgkit_x/Cargo.toml`: `name = "rpgkit_x"`, version and edition as today, the same external
  and `aegis_*` dependencies (the `../../../../engine/crates/...` paths keep their depth), and each dependency on
  another area written as an alias so the moved sources compile unchanged:
  `emberwake_y = { package = "rpgkit_y", path = "../rpgkit_y" }`. No dev-dependencies move (the integration tests
  stay).
- M4. The façade: `games/emberwake/crates/emberwake_x/Cargo.toml` keeps every line and adds
  `rpgkit_x = { path = "../../../../kits/rpg/crates/rpgkit_x" }`; a new `src/lib.rs` holds one doc line naming the
  kit crate and `pub use rpgkit_x::*;`.
- M5. `git mv games/emberwake/schema.json kits/rpg/schema.json`; `source.rs:31`
  (`include_str!("../../../schema.json")`) now resolves to it unchanged.
- M6. `emberwake_host`, `emberwake_gui`, tools, CI and docs are not touched.

Acceptance (rule-derived):

| Check | Expectation from |
| --- | --- |
| each file under `kits/rpg/crates/*/src/` and `kits/rpg/schema.json` hashes (`git hash-object`) to the `develop` blob of its old path | M2, M5 |
| `git diff --stat develop` lists no file under `tests/`, `parity/`, `data/`, `legacy/`, `emberwake_host/`, `emberwake_gui/` | M6, rule 2 |
| `cargo build --workspace --all-targets` without a warning; the gate green with every parity suite run | rules 3, 4 |
| the same passing test names before and after; the moved unit tests (`grid.rs`, `art.rs`) pass under `rpgkit_battle`, `rpgkit_content` | rule 4 |
| `cargo test -p emberwake_battle --test trace_parity` (and the other four parity script commands) run as before | KIT1 2.1 |
| `aegis` outputs of rule 5 unchanged; the GUI starts (`emberwake screen=runner`) | rule 5 |

Size: M (72 source files, about 30,600 lines, plus 13 manifests and six façade files).

## 3. KIT1-N: the kit names no game

- N1. In every kit `Cargo.toml` the alias keys become the real names (`rpgkit_y = { path = "../rpgkit_y" }`) and in
  every kit source `emberwake_y::` becomes `rpgkit_y::` (today 182 lines in 48 files; most are `use` lines).
- N2. `GAME_ID` leaves `rpgkit_world` (`lib.rs:21-22`) for the `emberwake_world` façade. Kit
  `Data::find_data_dir(start)`: the nearest of `start` and its ancestors that holds `game.json`, `data/catalog/` and
  `data/worlds/`, then its `data/`. New kit `Data::find_game_data_dir(start, id)`: today's rule
  (`aegis_content::manifest::find_game_dir(start, id)`, then the same folder checks). The façade adds
  `pub fn find_data_dir(start: &Path) -> Option<PathBuf>` = `Data::find_game_data_dir(start, GAME_ID)`; the two
  non-test callers use it: `emberwake_host/src/cli.rs:101` and `emberwake_gui/src/main.rs:64`. Every test call
  (from inside `games/emberwake/`) keeps calling `Data::find_data_dir` and finds the same folder.
- N3. The writer's hooks struct `store::Emberwake` (`store.rs:232-287`) becomes `store::RpgContent`;
  `emberwake_host/src/cli.rs:25` and `:47` follow.
- N4. `emberwake_host/src/lib.rs` moves to `kits/rpg/crates/rpgkit_host/src/lib.rs` without its `pub mod cli;`
  (`lib.rs:42`); the `emberwake_host` façade's `lib.rs` is `pub use rpgkit_host::*;` and `pub mod cli;`
  (`cli.rs:10` `use crate::{Battle, Host}` resolves through the glob); `main.rs` and `tests/replay.rs` stay.
- N5. Doc comments and panic messages in kit files that name the game or its paths say "the game's" instead
  (`source.rs:1-7`, `:33-36` becomes "kits/rpg/schema.json", `data.rs:5`, `:70`, `emberwake_data/src/lib.rs:3`,
  `battle/src/sim.rs:284`, `content/src/lib.rs:1`, `host/src/lib.rs:1-4`); the comment in
  `tools/sprites/variants.py:49` names `rpgkit_content::art`. No string that a test or recording compares changes;
  if one would, it stays and the report names it.
- N6. The unit-test literals `ashlands` and `tidewrack` in `rpgkit_content/src/art.rs:309-337` become neutral ids
  (`world_a`, `world_b`); `ash_brawler` becomes `mob_brawler`.
- N7. Façade manifests: dependencies only their tests use move to `[dev-dependencies]`; nothing else.

Acceptance:

| Check | Expectation from |
| --- | --- |
| `git grep -n -i emberwake -- kits/` finds nothing | N1, N2, N5 |
| no `Cargo.toml` under `kits/` names a path under `games/` | KIT1 3.2 dependency rule |
| `aegis content check` and `aegis sim duel 5 2`, run from the repository root and from `games/emberwake/`, print the same as before | N2, rule 5 |
| the same passing tests; every parity suite run | rules 3, 4 |
| `Data::find_data_dir` from a scratch folder `t/games/demo/` holding `game.json`, `data/catalog/`, `data/worlds/` returns `t/games/demo/data`; from `t/` it returns None; `find_game_data_dir(t, "demo")` returns it (new kit unit test, scratch folder) | N2 |
| `git grep -n -i ashlands -- kits/rpg/crates` finds only the sim lines KIT1-SIM removes | N6 |

Size: M (mechanical renames in 48 files and five small edits).

## 4. KIT1-SIM, KIT1-G and KIT1-P

**KIT1-SIM: the sample world leaves the kit sim.** The kit sim names no world and no flag. `rpgkit_sim` gets a
game description, for example `SimGame { default_world, story: Option<StoryRun>, usage: Vec<String> }` with
`StoryRun { world, branch_flag, branches }` (the builder may shape it; these are the values it carries):

- S1. The default world of `world=` and of the `extra=` check (`args.rs:16`, `:126`, `:371-372`).
- S2. The story run of the `campaign` mode: the world whose auto-start quest chain is the story (`world.rs:12`,
  `:23`), the choice flag that branches it and the branch labels (`world.rs:14`, `:62`; `campaign.rs:149-168`).
  Without a story run the `campaign` mode prints one line saying the game has none.
- S3. The usage lines that name them (`args.rs:625`, `:637`).
- S4. The `emberwake_sim` façade keeps `run(d, argv)` with today's signature and passes the Ashlands values
  (`"ashlands"`, `"ashlands.steel_to_gate"`, the labels and lines from S1-S3). `emberwake_host/src/cli.rs` is
  unchanged.

Acceptance: `sim_parity` and `sim/tests/rules.rs` green; `aegis sim campaign 5`, `aegis sim duel 5 2`,
`aegis sim party` and an `extra=` run print byte-identical lines before and after; `git grep -n -i ashlands --
kits/rpg/crates` finds nothing. The base-class and classic skill tables stay in the kit sim (KQ8 a). Size: S.

**KIT1-G: the layer check in the gate.** `tools/fc.sh layers` (and the same step in `check_project.ps1`), run by
`gate` after the tests and in under a second:

- G1. For each folder name under `games/`, `git grep -n -I -i -w <id>` over `engine/crates/`, `kits/*/crates/` and
  `kits/*/schema.json` finds nothing.
- G2. No `Cargo.toml` under `engine/` or `kits/` has a path into `games/`; none under `engine/` has a path into
  `kits/`.
- G3. The docs check's markdown list gains `kits/*/README.md`.
- Acceptance: the gate passes on `develop` after KIT1-SIM; a scratch commit adding `// emberwake` to a kit file, or a
  kit dependency on a game path, fails it with the file and line. Size: S. Layer: tools for every layer (row "kit").

**KIT1-P: the in-flight specs follow the path map** (design lane, after KIT1-M merges): DERIVE1, RULESET1 (with
`_INTENT`, `_TASKS`), ENCOUNTER2 (with `_TASKS`) and the new CAMPAIGN1, ROSTER1, STORYUI1 get their Touches and
module paths rewritten per section 5; no rule changes. The planner rewrites the TASKS rows' Touches the same way.

## 5. Path map (for every spec and task row after KIT1-M)

| Written today | After KIT1 |
| --- | --- |
| `games/emberwake/crates/emberwake_data/src/lib.rs` | `kits/rpg/crates/rpgkit_data/src/lib.rs` |
| `emberwake_rules/src/<m>.rs` (`r/<m>.rs`) | `kits/rpg/crates/rpgkit_rules/src/<m>.rs` |
| `emberwake_battle/src/<m>.rs` (`b/<m>.rs`) | `kits/rpg/crates/rpgkit_battle/src/<m>.rs` |
| `emberwake_content/src/{source,store,art}.rs`, `check/*` | `kits/rpg/crates/rpgkit_content/src/...` |
| `emberwake_world/src/<m>.rs` (`w/<m>.rs`) | `kits/rpg/crates/rpgkit_world/src/<m>.rs` |
| `emberwake_sim/src/<m>.rs` | `kits/rpg/crates/rpgkit_sim/src/<m>.rs` |
| `emberwake_host` "host lib" (RULESET1-V, ENCOUNTER2-H) | `kits/rpg/crates/rpgkit_host/src/lib.rs` after KIT1-N; `cli.rs` stays in the game |
| New modules in the specs (`r/ruleset.rs`, `r/derived.rs`, `b/round_intent.rs`, `b/round_intent_plan.rs`, `b/poise.rs`, `emberwake_sim/src/feel.rs`; STORYUI1's `emberwake_world/src/{story,board}.rs` and `check/story.rs`; the story-free world modules of CAMPAIGN1 and ROSTER1) | the kit crate of their area |
| `games/emberwake/schema.json` | `kits/rpg/schema.json` |
| A test that reads the shipped data or `parity/out` (`data_source.rs`, `saves.rs`, `phasefold.rs`, ...) | unchanged, in the `emberwake_*` façade's `tests/` |
| A new test on scratch data or its own frozen fixture (DERIVE1-R `tests/derived.rs`, ENCOUNTER2-H `tests/stay_behind.rs` if it builds its own data, RULESET1-X `timeline_golden.rs` on `fixtures/timeline_kit.json`) | `kits/rpg/crates/rpgkit_<x>/tests/` |
| `emberwake_gui/src/*` (STORYUI1's `story.rs`, `board.rs` included, KQ4), `data/**`, `tuning.json`, `assets/**` | unchanged |

## 6. Task rows (status `spec`; the planner writes `docs/TASKS.md`)

| Id | Title | Layer | Lane | Depends on | Spec | Touches |
| --- | --- | --- | --- | --- | --- | --- |
| KIT1-M | Kit crates by a byte-identical move: `rpgkit_{data,rules,battle,content,world,sim}` with crate aliases, the `emberwake_*` façades, the schema to the kit | kit + game | logic | - | `docs/specs/KIT1_TASKS.md` 2 | `Cargo.toml`; `kits/rpg/crates/**` (new); `kits/rpg/schema.json` (from `games/emberwake/schema.json`); `games/emberwake/crates/emberwake_{data,rules,battle,content,world,sim}/{Cargo.toml,src/**}` |
| KIT1-N | The kit names no game: real crate names, `GAME_ID` and the data-folder lookup to the game, hooks `RpgContent`, host lib to `rpgkit_host`, neutral comments and test ids | kit + game | logic | KIT1-M | `KIT1_TASKS.md` 3 | `kits/rpg/crates/**`; `games/emberwake/crates/emberwake_{world,host,data,rules,battle,content,sim}/{Cargo.toml,src/lib.rs}`; `emberwake_host/src/{lib,cli}.rs`; `emberwake_gui/src/main.rs`; `tools/sprites/variants.py` (comment) |
| KIT1-SIM | The sample world leaves the kit sim (default world, story run, usage lines passed by the game) | kit + game | logic | KIT1-N | `KIT1_TASKS.md` 4 | `kits/rpg/crates/rpgkit_sim/src/{lib,args,world,campaign}.rs`; `games/emberwake/crates/emberwake_sim/src/lib.rs` |
| KIT1-G | Layer check in the gate: no game named in engine or kit crates, no game path in their manifests; kit READMEs linted | kit (tools) | logic | KIT1-SIM | `KIT1_TASKS.md` 4 | `tools/fc.sh`, `check_project.ps1` |
| KIT1-P | In-flight specs on the kit paths (path map) | kit | design | KIT1-M | `KIT1_TASKS.md` 4, 5 | `docs/specs/{DERIVE1,RULESET1,RULESET1_INTENT,RULESET1_TASKS,ENCOUNTER2,ENCOUNTER2_TASKS}.md`, `games/emberwake/docs/specs/` or `docs/specs/` CAMPAIGN1, ROSTER1, STORYUI1 |
| KIT1-D | Docs delta (section 7) | kit + game | planner | KIT1-N (G for the reviewer and gate lines) | `KIT1_TASKS.md` 7 | the docs in section 7 |

Under option (a) the planner adds `KIT1-N` to the Depends on of DERIVE1-S, RULESET1-R, ENCOUNTER2-T and every later
kit-layer logic row, and `KIT1-SIM` to RULESET1-G (it adds the sim's `feel` mode). KIT1-M may run now beside
DERIVE1-E and RULESET1-E (engine files only). KIT1-SIM and KIT1-G may run beside DERIVE1-S (disjoint files).

## 7. Docs delta, HISTORY and ROADMAP rows

Docs the planner updates (builders report, KIT1-D applies):

- `docs/STRUCTURE.md`: the layers paragraph (kit code exists; the workspace members include `kits/*/crates/*`); a Kit
  section with rows for `kits/rpg/crates/rpgkit_*` (the module lists move from the game rows), `kits/rpg/schema.json`
  and new kit tests; the game rows become the façades (game code and Emberwake's tests); the Rules paragraph ("game
  rules live in the game's library crates" becomes "genre rules live in the kit crates; story mechanics in the game
  crates").
- `CLAUDE.md` Layers paragraph: the kit `kits/rpg/` holds the crates `rpgkit_*`, `schema.json`, `assets/`, `kit.json`;
  the game list loses `schema.json`.
- `README.md` (root) layout; `games/emberwake/docs/GAME_BRIEF.md` "where to look" rows (lines 11-18) and the art note
  (line 166, `rpgkit_content::art`); `engine/docs/PORT_NOTES.md` module paths; `kits/rpg/README.md` (crates, schema,
  the base classes as the kit's contract, the Ashlands as the sample and parity world on the classic ruleset; the
  "no code" sentence is replaced).
- `engine/docs/authoring/`: a game-agnostic `kit_layout.md` (a kit's folders: `<kit>kit_*` crates, schema, assets,
  `kit.json`, its sample world; extracting a kit from a game by a byte-identical move behind façades, then de-naming;
  the layer check), linked from `layer_rule.md` and the authoring README.
- `.claude/agents/change-reviewer.md` rule 8 (kit crates as well as engine crates; KIT1-G's check); skills
  `rust-conventions`, `engine-checklist`, `known-issues` (crate paths; a worktree without `parity/out` silently skips
  the parity suites), `planner` (Touches use kit paths).
- `games/emberwake/docs/lore/specs/rpg_kit_and_system_architecture.md`: the "Where it lives today" column of section
  1 and gap 1 point at the kit crates.
- `docs/EFFICIENCY.md`: the first build after KIT1-M and the gate time after it, measured.

HISTORY row (append):

| Date | Concept | Status | Replaced by | Why |
| --- | --- | --- | --- | --- |
| 2026-10-25 | Kit code only when a second game shares rules (SPLIT1 principle 2; `docs/STRUCTURE.md`, `kits/rpg/README.md`) | replaced | KIT1: the genre rules move into `kits/rpg/crates/rpgkit_*` in M0, the game crates become façades | User 2026-10-25 ("Worlds and reuse"): the new campaign is a second ruleset user inside the same game and the Ashlands become the kit's sample, so the reason to wait no longer holds |

ROADMAP row (`proposed` until the user answers KQ1-KQ8):

| # | Area | Status | Depends on | Notes |
| --- | --- | --- | --- | --- |
| KIT1 | Kit extraction (kit crates, the Ashlands as the kit's sample) | proposed | - | M0: genre rules into `kits/rpg/crates/rpgkit_*`, the `emberwake_*` crates become façades with game code and Emberwake's tests, the schema to the kit, a layer check in the gate; the Ashlands stay the kit's sample and parity world (classic); spec `docs/specs/KIT1.md`; KQ1-KQ8 open |
