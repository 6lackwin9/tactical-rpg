# KIT1: the RPG kit extraction (kit crates, the Ashlands as the kit's sample)

Status: proposed (spec 2026-10-25, from the user's approval of a kit extraction in M0, decisions "Worlds and reuse",
"Layer rule", "Ruleset placement", "Slice is data only" and "Class names per world" in
`games/emberwake/docs/decisions/lore.md`)
Depends on: none (the moves start from `develop` as it is; DERIVE1-E and RULESET1-E touch engine files only)
Lane: logic (M, N, SIM, G), design (P), planner (D)
Touches: per task in [KIT1_TASKS.md](KIT1_TASKS.md) section 5; overall `Cargo.toml`, `kits/rpg/crates/**` (new),
`kits/rpg/schema.json` (moved), `games/emberwake/crates/emberwake_{data,rules,battle,content,world,sim,host}/`
Companion: [KIT1_TASKS.md](KIT1_TASKS.md) (the steps with rules and acceptance checks, the path map for the in-flight
specs, task rows, docs delta)

## 1. Goal

The genre rules that every tactical RPG on Aegis could use leave the game crates and become the RPG kit's crates in
`kits/rpg/crates/`, named story-free. The game `games/emberwake/` keeps only what its story names, its tests and its
binaries. Behaviour does not change: every step is a move, the gate and the Godot parity suites stay green after each
one, and no test, golden trace or recording is edited. The Ashlands (classic ruleset) stay the kit's playable sample and
parity world; the new campaign "A Scarlet Lullaby" (`sunk_ward`, timeline ruleset) is then built on the kit crates.

History check (`docs/HISTORY.md`, SPLIT1): SPLIT1 principle 2 said kit code comes "when a second game shares rules"
(`docs/specs/SPLIT1.md` section 1, `docs/STRUCTURE.md` layers paragraph, `kits/rpg/README.md`). That reason no longer
holds: the user decided on 2026-10-25 that the rules move into kit crates now, with the Ashlands as the kit's sample
and the new campaign as the second ruleset user ("Worlds and reuse"). A HISTORY row records the change (KIT1_TASKS
section 7). Nothing in HISTORY is re-proposed.

## 2. Survey: the game crates today

Source lines counted 2026-10-25 on `develop` (487b59d). Dependency order: data, rules, battle, content, world, sim,
host, gui.

| Crate | Src lines | Modules | Depends on | Genre or story-named |
| --- | --- | --- | --- | --- |
| `emberwake_data` | 506 | `lib.rs`: `ClassDef`, `UnitDef`, `SkillDef`, `ItemDef`, `PassiveDef`, `Catalog` | `aegis_core` (`emberwake_data/Cargo.toml:7`) | genre; no story names |
| `emberwake_rules` | 2,240 | `stats`, `budget`, `gear`, `carry`, `unit`, `damage`, `status` (`lib.rs:4-10`) | `aegis_core`, data (`Cargo.toml:7-8`) | genre; one base-class fallback `"warrior"` (`unit.rs:519`) |
| `emberwake_battle` | 10,079 | `grid`, `path`, `world`, `skill`, `dojo`, `intent`, `log`, `metrics`, `report`, `command`, `resolver`, `plan`, `ai`, `encounter`, `session`, `sim` (`lib.rs:3-18`) | `aegis_core`, data, rules (`Cargo.toml:7-9`) | genre; no story names in code |
| `emberwake_content` | 8,704 | `art`, `source`, `store`, `check/` (15 files) (`lib.rs:5-8`) | `aegis_core`, `aegis_content`, rules, data, battle (`Cargo.toml:7-13`; battle for the grid and arrival constants, `check/encounter.rs:8-9`, `check/simple.rs:5`) | genre; compiles the schema in (`source.rs:31-37`); the writer's hooks struct is named `Emberwake` (`store.rs:232-287`); unit-test literals `ashlands`, `tidewrack` (`art.rs:309-337`) |
| `emberwake_world` | 5,678 | `battle`, `data`, `dialogue`, `explore`, `new_game`, `progression`, `purse`, `quest`, `rolls`, `save`, `shop`, `skill_text`, `state`, `travel` (`lib.rs:6-19`) | `aegis_core`, `aegis_content`, `aegis_save`, data, rules, content, battle (`Cargo.toml:7-13`) | genre; `GAME_ID = "emberwake"` (`lib.rs:21-22`) used by `Data::find_data_dir` (`data.rs:72-74`); save format `fc_save` v12 (`save.rs:17-18`) |
| `emberwake_sim` | 3,377 | `args`, `runner`, `variants`, `modes`, `party`, `scenarios`, `world`, `roles`, `role_mode`, `campaign`, `targets`, `kpi` (`lib.rs:6-17`) | `aegis_core`, data, rules, battle, world (`Cargo.toml:7-11`) | genre except the sample world and its story branch: `DEFAULT_WORLD = "ashlands"` (`args.rs:16`, `:126`, `:371-372`), `WORLD`, `STEEL_FLAG` (`world.rs:12-14`, `:23`, `:62`), the campaign mode (`campaign.rs:149-168`, `lib.rs:45`), usage text (`args.rs:625`, `:637`); classic class and skill ids in its tables (`args.rs:9`, `:30`, `:117-118`, `roles.rs:25-82`, `scenarios.rs:26-29`, `role_mode.rs:16`) |
| `emberwake_host` | 652 | `lib.rs` (the battle command set `Battle`, `Host`; declares `pub mod cli` at `lib.rs:42`), `cli.rs` (`Emberwake: Game`, `cli.rs:22-47`), `main.rs` (binary `aegis`, `main.rs:4`) | `aegis_core`, `aegis_host`, `aegis_cli`, `aegis_content` and all six library crates (`Cargo.toml:11-20`) | `lib.rs` genre; `cli.rs`, `main.rs` game |
| `emberwake_gui` | 11,345 | 18 files: screens (`menus`, `region`, `hud`, `shop`, `props`, `worldmap`, `hub`, `party`, `battle`), dev tools (`runner`, `forge`, `dojo`, `gallery`), `screens`, `intent_text`, `music`, `tuning`, `main` | every crate above plus `aegis_ui`, `aegis_save` (`Cargo.toml:11-21`) | screens genre-shaped but bound to the game context; start-up is game code (`main.rs:34-107`); test fixtures name the Ashlands (`hub.rs:498-532`, `music.rs:161-195`, `runner.rs:345-352`) |

Story-named code is small: the game id, the sim's sample world and branch flag, one hooks struct name, doc comments
naming `games/emberwake/` paths (182 lines with `emberwake` in 48 files of the six library crates and `host/lib.rs`,
almost all `use emberwake_<x>::` lines), and test fixtures. Everything else is genre code ported from Godot.

### 2.1 Binaries and tools

| Item | Where | After KIT1 |
| --- | --- | --- |
| `emberwake` (the GUI) | `emberwake_gui/Cargo.toml:6-8` | unchanged, game |
| `aegis` (CLI, `sim`, `battle`, `serve`, the content tool `aegis content`) | `emberwake_host/Cargo.toml:6-8`, `main.rs:4`; the content tool validates with the schema compiled into the content crate (`source.rs:31-37`) and writes through its hooks (`store.rs:282-287`) | unchanged binary in the game; schema and hooks come from the kit |
| `tools/fc.sh` | build and test `--workspace` (`fc.sh:5-6`, `:30`, `:44`); `aegis` = `cargo run -p emberwake_host` (`:80`); parity recording (`:93-106`) | unchanged; KIT1-G adds a `layers` check |
| `check_project.ps1` | `build` and `test --workspace` (`:36`, `:50`) | unchanged; KIT1-G mirrors the check |
| `run.bat` | `cargo build --release -p emberwake_gui -p emberwake_host` (`:193-199`) | unchanged |
| CI `.github/workflows/gate.yml` | Godot export (`:56`), then `tools/fc.sh gate` | unchanged |
| Parity scripts `games/emberwake/parity/*.sh` | each ends "then: `cargo test -p emberwake_<x> --test <suite>`" (`export.sh:3`, `trace.sh:3`, `journey.sh:3`, `sim.sh:4`, `checks.sh:4`) | unchanged, because the tests stay in the `emberwake_*` packages |

### 2.2 Tests, parity and golden files

| Suite | Reads | Location rule |
| --- | --- | --- |
| `emberwake_rules/tests/parity.rs`, `emberwake_battle/tests/trace_parity.rs`, `emberwake_sim/tests/sim_parity.rs`, `emberwake_world/tests/{journey,saves,world_traces,data_source,store}.rs`, `emberwake_content/tests/checks_parity.rs`, `emberwake_host/tests/replay.rs` | Godot recordings in `games/emberwake/parity/out/` (gitignored) through `CARGO_MANIFEST_DIR` + `../../parity/out` (e.g. `trace_parity.rs:14`, `data_source.rs:12-17`, which also reads `../../../../legacy/data`) | the Godot benchmark of Emberwake's data: stays in the game |
| `emberwake_world/tests/phasefold.rs` and `tests/fixtures/phasefold_golden.json` | the shipped data and the golden file (`phasefold.rs:26-30`; recorded once, never re-recorded) | stays in the game |
| `emberwake_world/tests/{dojo,history,intents,item_text,npc_services,report,rewards,shop_trade,skill_ranks}.rs`, `emberwake_content/tests/{art_looks,asset_checks,prop_checks,reward_checks}.rs`, `emberwake_sim/tests/rules.rs` | the shipped data via `Data::find_data_dir(CARGO_MANIFEST_DIR)` or `../../data` | stays in the game |
| In-module unit tests of the library crates (`grid.rs` 1, `art.rs` 2) | nothing on disk | move with their module |

Every integration test finds its files relative to its package folder, and the parity scripts name the packages. A
moved test would change those paths and names, so the tests do not move (section 3.2).

### 2.3 Workspace, build and gate

`Cargo.toml:3-6`: members `engine/crates/*` and `games/*/crates/*` (no kits). The gate (`tools/fc.sh gate`,
`check_project.ps1 -Fast -Quiet`) builds every target with warnings as errors and runs every test: 27 s warm, the
battle traces ~8 s, journeys ~7 s (`docs/EFFICIENCY.md:9`); a first build of a checkout takes a few minutes
(`docs/EFFICIENCY.md:10`). Parity suites skip themselves when `parity/out` is missing (`tools/fc.sh:4`), so every KIT1
step must gate in a checkout that has the recordings (KIT1_TASKS section 1).

## 3. Target layout

### 3.1 Kit crate names

Kit crates are named `rpgkit_<area>`: the kit id (`kits/rpg/kit.json` `id`) plus `kit`, then the same area names as
today. Why: (1) a `use rpgkit_battle::...` line states its layer, as `aegis_` and `emberwake_` do; (2) the prefix is
unique in a grep, where `rpg_` would also match ordinary identifiers; (3) "kit" alone already names the engine's
widget kit (`aegis_ui::ui`), so `rpgkit_` avoids that clash; (4) keeping the area names (`rules`, `battle`, ...) maps
every module one to one, so a builder finds `session.rs` where it was, under a new crate. Alternative: `rpg_*` (KQ1).

### 3.2 Crate and file map

| Today | Kit (`kits/rpg/`) | Stays in the game (`games/emberwake/`) |
| --- | --- | --- |
| `emberwake_data/src/lib.rs` | `crates/rpgkit_data/src/lib.rs` | façade `emberwake_data` |
| `emberwake_rules/src/*` | `crates/rpgkit_rules/src/*` | façade; `tests/parity.rs` |
| `emberwake_battle/src/*` | `crates/rpgkit_battle/src/*` | façade; `tests/trace_parity.rs` |
| `emberwake_content/src/*` incl. `check/` | `crates/rpgkit_content/src/*` | façade; five tests |
| `emberwake_world/src/*` | `crates/rpgkit_world/src/*` | façade with `GAME_ID` and the game's data-folder lookup (KIT1-N); 15 tests and the golden fixture |
| `emberwake_sim/src/*` | `crates/rpgkit_sim/src/*` | façade with the Ashlands story run of the `campaign` mode (KIT1-SIM); two tests |
| `emberwake_host/src/lib.rs` | `crates/rpgkit_host/src/lib.rs` (KIT1-N) | `cli.rs`, `main.rs` (binary `aegis`), `tests/replay.rs` |
| `emberwake_gui/**` | nothing in KIT1 (KQ4) | everything |
| `games/emberwake/schema.json` | `schema.json` (KQ2) | nothing until a game-only kind exists |
| `games/emberwake/tuning.json`, `data/**`, `assets/**`, `parity/**`, `playtests/**` | nothing (KQ3; kit assets are already in `kits/rpg/assets/`) | everything |

**The façade.** Each `emberwake_<x>` package keeps its name, folder, `Cargo.toml` dependencies and `tests/`; its
`src/lib.rs` becomes `pub use rpgkit_<x>::*;` plus the game's own items. A glob re-export carries modules and
re-exported items, so every path `emberwake_<x>::<module>::<Item>` still resolves to the same type: the GUI, the host
CLI, every test and every parity script compile and run unchanged. From then on the façades are the game crates: the
home of game code (a story mechanic nothing else could use) and of Emberwake's tests.

**Kit dependency rule.** A kit crate depends only on `aegis_*`, other `rpgkit_*` crates and external crates; never on
a path under `games/`; its code, manifests and schema name no game (id, title, data paths, the worlds and flags of a
game's story). Exception list: KQ8. Engine crates never depend on kit crates.

### 3.3 Data, schema, assets and the content tool

| Thing | Where after KIT1 | Notes |
| --- | --- | --- |
| Schema (kinds and fields, a Godot export copy) | `kits/rpg/schema.json`, compiled into `rpgkit_content` (`include_str!("../../../schema.json")`, the same text resolves to the kit file) | every kind today is genre (content: classes, units, skills, items, passives, encounters, terrain, item families; world: 14 kinds), and the slice's new fields are kit hooks ("Slice is data only"); a game extension schema comes with the first game-only kind (KQ2) |
| Catalog and worlds | `games/emberwake/data/` (catalog, `ashlands`, `iron_reaches`, `tidewrack`, later `sunk_ward`) | a kit data layer (the base classes in the kit) is KQ3 |
| Tuning | `games/emberwake/tuning.json` | the GUI's feel values |
| Assets | game, kit, engine layers as today (ASSETORG2) | unchanged |
| Content tool | `aegis content` (game binary): kit schema, kit hooks, game data folder | commands and output unchanged |

The Ashlands are the kit's sample and parity world by role: their data stays in the game, the parity suites that
replay them stay in the game façades, and the kit's docs name them as the sample (KQ3).

## 4. When: options relative to the slice

The slice's specs all edit the same hub files: `emberwake_battle` `session`, `plan`, `command`, `ai`, `resolver`;
`emberwake_world` `state`, `save`, `battle`; `emberwake_content` `source`, `store`, `check/*`; `emberwake_data`
`lib.rs`; `schema.json`; `emberwake_gui/battle.rs`. Today about 32 logic tasks are queued on them (DERIVE1 S, R;
RULESET1 R, S, T, IN, PL, P, K, V, AI, D, G, X; ENCOUNTER2 T, H, HA, W, K, L, X, F, R, RA; an estimated 8 more from
CAMPAIGN1, ROSTER1, STORYUI1); none has started: the only open logic branches are DERIVE1-E and RULESET1-E, engine
files only (`docs/AGENT_MAP.md:7-9`).

| Option | What | Cost now | Merge-conflict and rework cost | Risk |
| --- | --- | --- | --- | --- |
| (a) KIT1 first | M, N, SIM, G back to back (KIT1_TASKS), then the slice builds in the kit crates | 3 logic runs plus a small one (M ~45 min, N ~60 min, SIM ~30 min, G ~15 min); the first slice logic task waits for M and N (~2 runs); every worktree rebuilds once (a few minutes) | ~0 conflicts (no branch open on these files); rework: the Touches of DERIVE1, RULESET1, ENCOUNTER2 and the three new specs follow the path map (KIT1-P, mechanical) | lowest: a move with nothing in flight; the layer check gates the slice from its first task |
| (b) Slice first | the slice lands in the game crates; KIT1 after M2 | none now | later: the same moves over ~20-25% more code (new modules `ruleset`, `round_intent`, `round_intent_plan`, `poise`, `derived`, `feel`, the ENCOUNTER2 rooms) and more `use emberwake_*` lines for N; a review pass over ~32 merged tasks for story names in kit-tagged code, since no crate boundary or grep can catch them during the slice; a freeze window later while M2b and Act 2 work are open | layer creep in the slice; contradicts the M0 order of "Slice is data only" |
| (c) Thin step now | M now (byte-identical move with crate aliases); N and SIM later | 1 run now | M itself never conflicts (pure renames rebase cleanly); N grows with every slice task that adds `use emberwake_*` to kit files and conflicts with every branch open when it runs; the kit names the game (aliases) for the whole slice, so G cannot gate | N slips indefinitely; two disruptions instead of one |

**Recommendation: (a).** Dispatch KIT1-M now beside DERIVE1-E and RULESET1-E (disjoint files), then KIT1-N; make
DERIVE1-S, RULESET1-R, ENCOUNTER2-T and every later kit-layer logic task depend on KIT1-N. KIT1-SIM and KIT1-G can
run beside DERIVE1-S (disjoint files: the sim crate and the tools). Expected cost: about 2.5 logic-lane hours and one
rebuild per worktree; no conflict expected. If a game-crate branch is opened before M lands, M still rebases cleanly
(every source file is a 100% rename) and N costs a few `use`-line hunks.

## 5. Tests and parity

- No integration test, fixture, golden file or recording moves or changes: they stay in the `emberwake_*` façades,
  which keep their package names and folders, so `CARGO_MANIFEST_DIR` paths and the parity scripts' `-p` names hold.
- The in-module unit tests move with their modules and run under the kit crate names (`cargo test --workspace`).
- KIT1-N changes no printed string a test or recording compares; KIT1-SIM keeps the sim's printed lines byte-identical
  (`sim_parity`, `sim/tests/rules.rs`, and a before/after diff of `aegis sim` output).
- Each step's acceptance includes the same set of passing tests before and after (KIT1_TASKS section 1).
- Retired parity checks: none. A pure move changes no behaviour, so no HISTORY row for checks is needed; the only
  HISTORY row is the dropped "kit code waits for a second game" principle (KIT1_TASKS section 7).
- New kit tests (from the slice on) live in `kits/rpg/crates/<crate>/tests/` and use scratch data only; a test that
  needs the shipped data or `parity/out` belongs to the game façade's `tests/`.

## 6. Out of scope

- Any behaviour change, rule fix or clean-up found on the way (each becomes its own task).
- The GUI screens (KQ4), a kit data layer and moving the classic catalog or the Ashlands (KQ3), a game extension
  schema (KQ2), per-world class display names (decision "Class names per world"; the roster or campaign spec owns it).
- The new campaign's save format (decision "Worlds and reuse"): CAMPAIGN1 or ROSTER1; the kit keeps `fc_save` v12.
- `kit.json` keys, the asset layers, the engine crates, `legacy/`, the parity recordings.
- Renaming modules inside the kit (`rpgkit_world` keeps the name `world`; a rename is a later clean-up if wanted).

## 7. Open questions for the user

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| KQ1 | Kit crate prefix | (a) `rpgkit_*`; (b) `rpg_*` (the kit id alone, like the game's `emberwake_*`) | (a): states the layer, unique in a grep, no clash with the engine's widget kit |
| KQ2 | Where the schema lives | (a) `kits/rpg/schema.json`, the kit owns every kind and field; a game extension file and an engine merge come with the first game-only kind; (b) it stays in the game and the kit reads it at run time (changes the loader's signatures; not a move) | (a): every kind is genre today and the slice's new fields are kit hooks |
| KQ3 | Kit data and the sample's home: the base classes "live in the kit" (decision "Class names per world") | (a) KIT1 makes them kit-owned by contract (the kit's README lists the six base classes; the rows stay in the game's catalog, the Ashlands stay in `games/emberwake/data/`); a later KIT2-DATA gives the engine data layers (kit catalog under the game's) and moves the base classes and the classic catalog into `kits/rpg/data/` after the slice; (b) build the data layer and move them in M0, before the slice | (a): the slice needs the names per world, not the file's location; a data layer is an engine feature with parity risk for the catalog export |
| KQ4 | The GUI screens | (a) stay in `emberwake_gui`; a KIT2-UI spec moves the genre screens after the slice's UI tasks (STAGE1, FX1, RULESET1-U, ENCOUNTER2-U); the slice's story-free screens (STORYUI1's `emberwake_gui/src/{story,board}.rs`) stay in the GUI crate, marked for KIT2-UI; (b) move the screens in KIT1; (c) as (a), but new kit screens start an `rpgkit_ui` crate on the engine's generic `Cx<G>` now | (a): 11,345 lines that are the hub of every open UI task, and STORYUI1 already plans its screens there |
| KQ5 | The façades long term | (a) keep `emberwake_*` as the game's crates: game code and Emberwake's tests; merge thin ones later if wanted; (b) collapse them into one game crate in KIT1-N (moves every test, changes the parity scripts' `-p` names and paths) | (a): the tests and parity harness stay byte-identical |
| KQ6 | Timing (section 4) | (a) KIT1 first; (b) slice first; (c) thin step now | (a) |
| KQ7 | Enforce the layers in the gate | (a) KIT1-G: `tools/fc.sh layers` in the gate (engine and kit name no game, depend on no game path; the engine on no kit); (b) the change-reviewer's rule only | (a): a grep costs under a second and catches creep at commit time |
| KQ8 | Ids of the kit's base content in kit code: the base-class fallback `"warrior"` (`unit.rs:519`) and the sim's tables of the six classes and their classic skills (`args.rs:9`, `:30`, `roles.rs:25-82`, `scenarios.rs:26-29`) | (a) allowed as the kit's base-class contract, listed in the kit README, turned into data with KIT2-DATA; (b) KIT1-SIM lifts them into the game | (a): they are genre ids, not story names, and the base classes belong to the kit |
