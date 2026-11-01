# CLEANUP: housekeeping proposals (housekeeper agent)

Newest pass on top. Candidate statuses: `proposed`, `confirmed (user <date>)`, `rejected (user <date>): <reason>`, `done`. Only the user confirms; confirmed candidates become `docs/TASKS.md` rows. Cleanup is never folded into a feature task: "after X" = run once X is merged, "superseded by X" = X deletes it anyway.

## Pass 3: Godot parity off, classic as data, code bloat (2026-11-01, develop at 4efb30c)

Confirmed by the user 2026-11-01, all as recommended (decision "Godot parity retired and cleanup pass 3 answers"); rows CLN-36 to CLN-53 in docs/TASKS.md.

Scope (user 2026-11-01, via the planner): A. retire the Godot parity checks ("legacy Godot code can sit in legacy/ but
legacy should not be part of the test builds anymore"); B. classic special cases (decision "INPUT1-M rule choices and
the classic ruleset": classic is one ruleset template over the rule library, not a separate code path); C. other
cleanup worth starting now. Read-only scan of the planner checkout (detached `origin/develop`), the desktop main
checkout's `games/emberwake/parity/out` (sizes only), `git log`, `tools/fc.sh docs`. Supporting decisions: "Port goal
and the end of Godot (user 2026-10-20)" clause (2) ("after the PORT rows are done, Godot is no longer a reference"; no
PORT row is open in TASKS) and "Saves and data edits (user 2026-11-01)" ("long-term save compatibility is not a goal").
Every candidate is `proposed`. Lanes: logic-builder (code, tests, tools), asset-builder (data through `aegis content`),
planner (TASKS, ROADMAP, decisions, HISTORY, `.claude/*`, CLAUDE.md), design-brainstorm (spec and lore doc splits).

### Pass 3 ranked summary

| Rank | Id | Scope | What | Size | Lane | Layer | When | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | CLN-36 | A | Tests that use the Godot export only as data: switch to the shipped data (replay, ruleset_requests); drop three one-off "frozen data reports nothing new" checks and timeout_verdict | S | logic-builder | game | now | done (2026-11-01) |
| 2 | CLN-37 | A | Writer resave check on the shipped data (every file re-saves byte for byte) instead of Godot's resave and its five AFTER_GODOT lists | S | logic-builder | game | now | done (2026-11-01) |
| 3 | CLN-38 | A | RNG and hash goldens into `aegis_core`; unit and damage goldens on a frozen catalog (the rules' only direct tests) | S-M | logic-builder | engine + game | now | done (2026-11-01) |
| 4 | CLN-42 | A | Switch Godot off: delete the pure-Godot suites, `games/emberwake/parity/`, `fc.sh parity`, the CI Godot steps, the hook's Godot install, the export-only CLI keys | M | logic-builder | game + kit + tools | after CLN-36..41 | proposed |
| 5 | CLN-43 | A | Docs and rules: CLAUDE.md "Godot parity", decisions, HISTORY, skills, agents, workflow docs, PORT_NOTES | S-M | planner (+ design-brainstorm for PORT_NOTES) | process | with CLN-42 | proposed |
| 6 | CLN-47 | C | Save migrations v1-v12 and the Godot JSON text emulation (decision "Saves and data edits") | S | logic-builder | kit | with CLN-42 (or before) | proposed |
| 7 | CLN-49 | C | 20 executed seeds out of `data/seeds/` (the seeds README rule) | S | planner (git rm) | game | now | proposed |
| 8 | CLN-39 | A | Sim parity: convert to an engine golden on the frozen catalog, or delete (Q-A2) | S | logic-builder | game | now | done (2026-11-01) |
| 9 | CLN-44 | B | Classic as a `rulesets` row: no hard-coded `Ruleset::classic()`, no closed id lists, one fallback | M | logic-builder + asset-builder | kit + game | after INPUT1-U | proposed |
| 10 | CLN-45 | B | Split the legacy-vs-timeline switches bundled under `round_intent` into named rule options | M-L | logic-builder | kit | after CLN-44, INPUT1-U, INPUT1-L | proposed |
| 11 | CLN-46 | B | Screens and host read rules, not `is_classic()` | S | logic-builder | game + kit | after CLN-44 | proposed |
| 12 | CLN-48 | C | The `wave_*` fold shim (PHASEFOLD 1.4) once nothing reads Godot data | S | logic-builder | kit | after CLN-42 | proposed |
| 13 | CLN-50 | C | Manual planning mode (MM1), superseded by picks; no screen sets it since INPUT1-U slice 2 | S | logic-builder | kit + game | after INPUT1-U | proposed |
| 14 | CLN-40 | A | Content checks parity: slim engine golden of the broken-entry cases, or delete (Q-A3) | M | logic-builder | game | now | proposed |
| 15 | CLN-41 | A | Journey parity: engine random-walk golden on the shipped data, or delete (Q-A4) | M | logic-builder | game | now | proposed |
| 16 | CLN-51 | C | `deferred_plan` in the battle screen rebuilds the host's round_intent handling (review finding) | S | logic-builder | kit + game | after INPUT1-U (superseded if slice 3 takes it) | proposed |
| 17 | CLN-53 | C | Oversized docs: archive built STAGE1 and WINDUP1; split CANON, STORY_GUIDE; split INPUT1, ROSTER1, CAMPAIGN1 once their open rows land | S-M | design-brainstorm | docs | now / after | proposed |
| 18 | CLN-52 | C | `emberwake_gui/src/battle.rs` (10,148 lines; ~4,100 of them 16 test modules): move tests out, then split the screen | M-L | logic-builder | game | at a flush point after INPUT1-U, -T, -PB, -AP, VISCLAR1-A | proposed |

Questions for the user: see "Pass 3 questions" (Q-A1 to Q-A6, Q-B1, Q-B2, Q-C1, Q-C2).

### Pass 3 scope A inventory: everything that compares against Godot or reads legacy/

Is `legacy/` compiled? No. The workspace members are `engine/crates/*`, `kits/*/crates/*`, `games/*/crates/*`
(Cargo.toml:3-7); `legacy/` holds no `Cargo.toml` or `.rs` file and no `include_str!`/`include_bytes!` names it. It is
read at test run time in two ways: (1) four game test files open `legacy/data` directly (checks_parity.rs:98, 141, 161;
data_source.rs:17; campaign_checks.rs:13; story_checks.rs:18); (2) twelve test files read
`games/emberwake/parity/out` (gitignored, 51 MB on the desktop, recorded by Godot 4.7.2 running `legacy/` through
`games/emberwake/parity/*.sh`). CI installs Godot, imports `legacy/` and runs `export.sh` before every gate
(.github/workflows/gate.yml:34-56); the cloud session hook installs Godot and imports `legacy/`
(.claude/hooks/session-start.sh:3, 76-97). "Removing legacy from test builds" concretely means: no test opens
`legacy/` or `parity/out`, no gate or CI step runs Godot, `tools/fc.sh parity` and `games/emberwake/parity/` go,
`legacy/` stays on disk untouched (28 MB) and the `legacy` branch stays protected (branch-cleanup.yml:51, keep).

Two of today's tests fail, not skip, on a checkout without `parity/out`: `emberwake_host/tests/replay.rs:11`
(`expect("missing catalog.json ...")`) and `emberwake_rules/tests/parity.rs:19, 160, 188` (`expect`/`panic`); CI is
green only because it runs `export.sh` first. So CLN-36 and CLN-38 must land before the CI Godot step goes (CLN-42).

| Item | File:line | Compares with | What it protects beyond Godot today | Recommendation |
| --- | --- | --- | --- | --- |
| Battle traces | `emberwake_battle/tests/trace_parity.rs` (91 lines) | Godot balance-sim fights (traces 12 MB) | Classic battle determinism step by step on the frozen catalog | (1) delete: the engine's own `phasefold_golden.json` (72 classic fights, 4 presets, morale off and on, 3 seeds, step for step) and the inert-extensions check already guard classic battles; Q-A1 for the pouch-item traces |
| World battles | `emberwake_world/tests/world_traces.rs` (153) | Godot WorldEncounter traces (7.5 MB) | World roster, guests, objectives, escape, hold tiles, start buffs, morale | (1) delete; phasefold golden covers two world encounters, the ENCOUNTER2 and HOOKS1 kit tests cover hold, escape and phases (Q-A1) |
| Journeys | `emberwake_world/tests/journey.rs` (241) | Godot game-loop journeys (13.5 MB) | The only whole-loop regression (explore, dialogue, shop, quests, saves, screen views) | (2) convert, CLN-41, or (1) delete (Q-A4) |
| Saves | `emberwake_world/tests/saves.rs` (67) | Godot's decode of 10 save fixtures v1-v10 | Migrations v1-v10, and a write/read round trip | (1) delete with CLN-47 (round trips are tested by `rpgkit_world/tests/{campaign,roster,story,deck,travel_modes}.rs`); if Q-C1 keeps migrations, (2) commit the 10 fixtures (44 KB) with engine-recorded states |
| Writer resave | `emberwake_world/tests/store.rs:121-170` and the AFTER_GODOT lists :14-57 | Godot's ContentCatalog.save_all output | Byte-identical re-save (the writer's storage form) | (2) convert, CLN-37: re-save every shipped file, compare with itself |
| Loader | `emberwake_world/tests/data_source.rs` (333; AFTER_GODOT 57 rows :38) | Godot catalogs built from `legacy/data` | Loader output shape and key order, the schema copy | (1) delete: every game test loads the shipped data through the same loader, `store.rs` `shipped_data_checks_clean` checks it; a loader golden on live data would churn on every data edit |
| Content checks | `emberwake_content/tests/checks_parity.rs` (228; LIST_ITEMS_AFTER_GODOT :34) | Godot ContentSchema messages, 6,721 broken-entry cases (6.5 MB) | Message text of every old-kind field check (units, skills, items, encounters); no kit test covers those kinds this broadly | (2) convert slim, CLN-40, or (1) delete (Q-A3) |
| Sim | `emberwake_sim/tests/sim_parity.rs` (41) | 20 Godot sim command lines (108 KB) | Sim output tables (BAL2 modes) | (2) convert, CLN-39, or (1) delete (Q-A2); `emberwake_sim/tests/rules.rs` and the kit's feel/deadlines tests stay |
| Rules | `emberwake_rules/tests/parity.rs` (206) | Godot unit snapshots (124 KB), DamageCalculator cases (4.3 MB), RNG draws (4 KB), hash (5 KB) | RNG and hash sequences (every golden depends on them); the only direct tests of `rpgkit_rules` damage and unit building (no `#[test]` in rpgkit_rules/src) | (2) convert, CLN-38 |
| Timeout verdict | `emberwake_battle/tests/timeout_verdict.rs` (71) | none (uses the export as data) | ENCOUNTER2-H S2: the verdict moved from the screen into `Session::final_outcome` (a one-off move check) | (1) delete, CLN-36 (Q-A5: or point it at the shipped catalog) |
| Host replay | `emberwake_host/tests/replay.rs` (104) | none (export as data) | Command-log replay determinism | (3) keep the test, switch its data, CLN-36 |
| Host rulesets | `emberwake_host/tests/ruleset_requests.rs` (50) | none (export as a catalog without ruleset rows) | `new` picks a ruleset; a ruleset without its row is refused | (3) keep, switch to the shipped catalog with the `rulesets` rows removed in memory, CLN-36 |
| Campaign on frozen data | `emberwake_content/tests/campaign_checks.rs` (37) | none (`legacy/data`) | "the campaign checks add nothing on data without campaign fields" (one-off, CAMPAIGN1-S) | (1) delete, CLN-36 (the kit's `campaign_schema.rs` and `shipped_data_checks_clean` cover it) |
| Story on frozen data | `emberwake_content/tests/story_checks.rs:17` (first test) | none (`legacy/data`) | Same, for the story checks (STORYUI1-S TS2) | (1) delete that test, CLN-36; keep `shipped_manifests_resave_byte_identical` (:57) until CLN-37 covers it, then delete it too |
| Timeline golden | `emberwake_battle/tests/timeline_golden.rs`, `fixtures/timeline_golden.json` (1.9 MB), `timeline_kit.json` (0.5 MB) | none: the engine's own golden (RULESET1-X) on a frozen catalog | Timeline rules step for step; inert extensions on the Ashlands classic | (3) keep, engine-owned; its "inert extensions" half compares classic with classic plus extensions, not with Godot |
| PHASEFOLD golden | `emberwake_world/tests/phasefold.rs`, `fixtures/phasefold_golden.json` (3.2 MB) | none: engine-recorded before PHASEFOLD | Classic battles with arrival phases on the shipped data | (3) keep, engine-owned (it becomes the classic battle golden); its legacy-row fold tests (:207-220) go with CLN-48 |
| Sim help text | `rpgkit_sim/src/args.rs:9, 71, 110, 116, 430, 437`; `rpgkit_sim/tests/feel.rs:69, 137` | Godot's usage lines (via sim_parity) | Nothing: the engine-only keys (`feel`, `deadlines`, `ruleset=`, `rules=`, `rewards`, `tactic=`, `party=`, `armor=`, `tune=`) are hidden from the help only for parity | (1) list them in the help with CLN-42 (or CLN-39 if converted, re-recorded after) |
| Export loaders | `rpgkit_world/src/data.rs:90-98` (`Data::load` from an export folder); `rpgkit_data/src/lib.rs:636` (`Catalog::load_resolved`); `emberwake_host/src/cli.rs:8-9, 62-90, 111-124` (`catalog=`, `$AEGIS_CATALOG`, `sim parity=`) | Godot exports | Nothing once no export exists (no tool, doc or script passes `catalog=` or `parity=`) | (1) delete with CLN-42, unless CLN-38/39 read their frozen catalog through them (then keep `Catalog::load_resolved` only) |
| GUI shot folder | `emberwake_gui/src/main.rs:125` (`shot_dir: home.join("parity/out")`) | none | Where agent screenshots land | (1) point at a gitignored scratch folder (for example `games/emberwake/.shots/`) with CLN-42 |
| Recorders | `games/emberwake/parity/{export,trace,journey,sim,checks}.sh`, `godot/{export,trace,journey,checks,combat_shot,screen_tour}.gd` (1,589 lines) | Godot | Nothing else (SCREENS.md:8 names screen_tour.gd as the Godot UI tour) | (1) delete with CLN-42 (git history and the `legacy` branch keep them) |
| Gate and CI | `tools/fc.sh:4, 14-16, 40, 149-166`; `check_project.ps1:3-5, 57`; `gate.yml:2-6, 25, 34-56` | Godot | `fc.sh` line 166's "is a Godot command" hint (keep, it points at `legacy/tools/fc.sh`) | (1) drop the `parity` command, the summary greps "match Godot", the Godot env, cache, install and export steps |
| Session hook | `.claude/hooks/session-start.sh:3, 76-97` (20 Godot lines) | Godot | Nothing else | (1) drop the Godot install and the `legacy/` import (planner lane: `.claude/*`); the cloud setup script in the project files (outside the repo) needs the same edit by the user |
| Rule docs | CLAUDE.md "Godot parity" and line 3; skills, agents, `.agents/rules/engine_workflow.md`; list in CLN-43 | - | - | rewrite as an engine-golden rule, CLN-43 |
| Godot quirks in code | `rust-conventions` line 41 (roundi, JSON floats, murmur3 `hash()`, PCG32 `GodotRng`, `godot_sort`) | - | They are now simply the engine's semantics: every golden and the shipped balance depend on them | (3) keep the behaviour; the "reproduce Godot quirks" rule becomes "keep the engine's numeric semantics" (Q-C2 for renaming `GodotRng` etc.) |

Savings (estimates, not measured; measure the first gate after CLN-42 and add an EFFICIENCY row): desktop full gate
with `parity/out` present loses about 25-35 s of test time (EFFICIENCY.md:9, 15: battle traces ~8 s, journeys ~7 s,
world battles ~5 s, sim runs ~6 s; plus checks, loader and resave, a few s); up to 9 fewer integration-test binaries
to link (roughly 10-20 s on a build that touches the game crates, nothing on a warm no-change build); CI loses the
Godot install (cached), the `legacy/` import and `export.sh` (roughly 1-2 min of the 4 min 49 s `checks` job,
EFFICIENCY.md:36) plus the parity suites it ran; each new desktop worktree stops copying 51 MB (desktop.md:15); five
known-issues rows about stale or missing recordings retire (known-issues lines 23-25, 44, 58). Converted goldens add
about 1-3 MB of committed fixtures in total (CLN-38 to CLN-41).

### CLN-36

Tests that use the Godot export only as data. Scope A. Status: done (2026-11-01; the shipped catalog is opened
through `emberwake_world::Data::open`, already a dev-dependency, instead of `emberwake_content::source::load`). Size S. Lane logic-builder. Layer game.

- Where: `emberwake_host/tests/replay.rs:9-12` (fails without the export), `emberwake_host/tests/ruleset_requests.rs:13-16,
  27-30`, `emberwake_battle/tests/timeout_verdict.rs` (whole file), `emberwake_content/tests/campaign_checks.rs` (whole
  file), `emberwake_content/tests/story_checks.rs:17-55` (the first test).
- Why: none of them compares with Godot; they borrow the export or `legacy/data` as a data source, or re-check a one-off
  migration ("the new checks add nothing on frozen data", "the verdict moved from the screen") that landed weeks ago.
- Change: replay.rs reads the shipped catalog (`emberwake_content::source::load` on the found data folder; its
  encounters `goblin_ambush` exist there); ruleset_requests.rs builds that catalog and removes the `rulesets` rows in
  memory (its point is "a catalog without ruleset rows"); delete timeout_verdict.rs (Q-A5), campaign_checks.rs and
  story_checks.rs' first test.
- Risk: replay's expectations are self-consistent (record then replay), so a data source change cannot break them;
  ruleset_requests' first encounter id changes (it takes `keys().next()`). Covered by the tests themselves.
- Touches: `games/emberwake/crates/emberwake_host/tests/{replay,ruleset_requests}.rs`,
  `games/emberwake/crates/emberwake_battle/tests/timeout_verdict.rs`,
  `games/emberwake/crates/emberwake_content/tests/{campaign_checks,story_checks}.rs`. Dependencies: none. Spec pointer:
  `docs/specs/CLEANUP.md#CLN-36`.

### CLN-37

Writer resave on the shipped data. Scope A. Status: done (2026-11-01; as built: `every_shipped_file_resaves_byte_for_byte` in
`emberwake_world/tests/store.rs` walks every JSON file of `games/emberwake/data` and compares it with the writer's text
(`resave_text` for base, game-data and world kinds, `world_manifest_text` untouched for manifests), read only, no
scratch copy needed; 53 files; `seeds/` and `progression/xp_curve.json` skipped (no writer kind); nine world files not
in writer form (fields added since their last save, defaults only) listed in `PENDING_RESAVE` for an asset-lane
resave (CLN-37A, 2026-11-01: resaved, list removed, every file compared clean); story_checks.rs deleted whole, its one test covered). Size S. Lane logic-builder. Layer game.

- Where: `emberwake_world/tests/store.rs:14-57` (AFTER_GODOT 20, ITEMS_AFTER_GODOT, MANIFEST_AFTER_GODOT 8,
  QUEST_AFTER_GODOT 4, WORLD_KINDS_AFTER_GODOT 6, `at_freeze`), `:100-114` (`godot_encounters_folded`), `:117-170`
  (`writer_saves_exactly_what_godot_saves` on `parity/out/resave`).
- Why: the value worth keeping is "re-saving any data file without changes writes it back byte for byte" (the header,
  :1-3); today it is only checked against Godot's resave of `legacy/data`, so the live data's own form is unchecked
  except world manifests (story_checks.rs:57) and per-kind kit fixtures. Every new field adds a row to these lists
  (HISTORY 2026-10-21 to 2026-11-01: eight "parity narrowed" rows).
- Change: one test that re-saves every catalog, world and game kind of a scratch copy of `games/emberwake/data/` and
  compares each file with its original; delete the Godot test, the five lists, `at_freeze` and
  `godot_encounters_folded`; then delete story_checks.rs' `shipped_manifests_resave_byte_identical` (covered).
- Risk: a shipped file not in writer form fails at once; that is a real finding (fix with `aegis content` resave,
  asset lane). Touches: `games/emberwake/crates/emberwake_world/tests/store.rs`,
  `games/emberwake/crates/emberwake_content/tests/story_checks.rs`. Dependencies: CLN-36 (same story_checks file).
  Spec pointer: `docs/specs/CLEANUP.md#CLN-37`.

### CLN-38

RNG, hash, unit and damage goldens. Scope A. Status: done (2026-11-01; as built: the engine golden
`engine/crates/aegis_core/tests/rng_hash.rs` with `tests/fixtures/rng_hash.json` (11 KB: the six seeds of rng.json,
each with six rounds of `randi`, `randf`, `randi_range(-1, 1)`, `randi_range(0, 99)`, `randf_range(0.5, 2.0)`, their
bounds written in each row; hash.json's 60 roll keys, 4 strings and 5 ints, the string `ashlands:warden_keep|3|0`
replaced by the game-agnostic `region:keep|3|0`); the game golden
`games/emberwake/crates/emberwake_rules/tests/rules_golden.rs` with `tests/fixtures/rules_golden.json` (310 KB: 125 unit
snapshots, every catalog unit and every class at levels 1-10 and at each other tier at level 5; 1,652 damage cases,
each pool attacker against two targets with the hand, range, forced and rolled crit, skill, guard, cover and armour
pierce branches, plus the stance matrix on the level 5 classes) on the reused frozen catalog
`emberwake_battle/tests/fixtures/timeline_kit.json`; both recorded with `RECORD=1`; the recording matched Godot's rng.json
and hash.json on every shared input). Size S-M. Lane logic-builder. Layer engine + game.

- Where: `emberwake_rules/tests/parity.rs` (206 lines: `unit_snapshots_match_godot` :64, `damage_cases_match_godot`
  :117, `rng_matches_godot` :157, `godot_hash_matches` :185); `kits/rpg/crates/rpgkit_rules/src/lib.rs:2` ("checked
  against Godot output by tests/parity.rs").
- Why: the RNG and the hash decide every recorded golden (timeline, PHASEFOLD); `rpgkit_rules` has no `#[test]` of its
  own, so the damage and unit-building formulas are tested directly only here. These need no Godot once recorded.
- Change: (a) engine: `engine/crates/aegis_core/tests/rng_hash.rs` with `tests/fixtures/rng_hash.json` (seeds, draw
  kinds, values; hash inputs and values) recorded once from the engine (`RECORD=1`); the inputs come from today's
  rng.json and hash.json (8 KB). (b) game: `emberwake_rules/tests/rules_golden.rs`: unit snapshots and a slim sample of
  damage cases (aim under 300 KB: every class and tier once, each damage branch: crit, guard, armour, ranged, off hand)
  on a frozen catalog, recorded once from the engine; re-record only for an intended rule change named in HISTORY (the
  timeline golden's rule). Frozen catalog: reuse `emberwake_battle/tests/fixtures/timeline_kit.json` if it serves, else
  freeze one the same way (Q-A6).
- Risk: none for behaviour (record from the current engine). Touches: `engine/crates/aegis_core/tests/` (new),
  `games/emberwake/crates/emberwake_rules/tests/{parity.rs (delete), rules_golden.rs (new), fixtures/ (new)}`,
  `kits/rpg/crates/rpgkit_rules/src/lib.rs` (doc line). Dependencies: none. Spec pointer: `docs/specs/CLEANUP.md#CLN-38`.

### CLN-39

Sim parity. Scope A. Status: done (2026-11-01; as built: `emberwake_sim/tests/sim_golden.rs` with
`tests/fixtures/sim_golden.json` (82 KB, 22 runs, 526 lines: the 20 command lines of `sim.sh`, run 11 now naming both
world encounters with `world:` so it fights them instead of printing an argument error, plus `feel 5 1
enc=tl_duel_windup,tl_skirmish party=both` and `deadlines`) on the timeline golden's frozen catalog `timeline_kit.json`
and `tests/fixtures/sim_data.json` (110 KB: the shipped worlds and game data frozen from the loader output); recorded
with `RECORD=1`; the sim help now lists the `feel` and `deadlines` modes and the keys `ruleset=`, `rules=`,
`melee_diagonal=`, `rewards=`, `seeds=`, `tactic=`, `party=`, `armor=`, `tune=`, `losses=` (so CLN-42's help item is done);
`sim_parity.rs` deleted; passes with `parity/out` absent). Size S. Lane logic-builder. Layer game. Depends on Q-A2.

- Where: `emberwake_sim/tests/sim_parity.rs` (41 lines), `games/emberwake/parity/sim.sh` (the 20 RUNS).
- Why: compares 20 sim command lines with Godot's printout. Beyond Godot it is the only check of the sim's printed
  tables for the BAL2 modes (`rules.rs` checks argument errors, bands and sweeps).
- Change (convert): `emberwake_sim/tests/sim_golden.rs` with the 20 command lines and their engine output in
  `tests/fixtures/sim_golden.json`, on the frozen catalog of CLN-38 plus frozen worlds and game data (one fixture),
  recorded once; the help lines then list the engine-only keys (part of the recording). Or delete (Q-A2).
- Risk: output changes with every rule change; a golden on frozen data churns only on rule changes. Touches:
  `games/emberwake/crates/emberwake_sim/tests/{sim_parity.rs (delete), sim_golden.rs, fixtures/}`,
  `kits/rpg/crates/rpgkit_sim/src/args.rs` (help lines), `kits/rpg/crates/rpgkit_sim/tests/feel.rs:69, 137`.
  Dependencies: CLN-38 (frozen catalog). Spec pointer: `docs/specs/CLEANUP.md#CLN-39`.

### CLN-40

Content checks parity. Scope A. Status: proposed. Size M. Lane logic-builder. Layer game. Depends on Q-A3.

- Where: `emberwake_content/tests/checks_parity.rs` (228 lines; 4 tests :91, :135, :155, :196), reads
  `legacy/data` and `parity/out/checks.json` (6,721 cases, 6.5 MB).
- Why: beyond Godot it is the broadest check of the old kinds' field validation (message for message); the kit's
  check tests cover the newer kinds (rulesets, hooks, story, campaign, roster) but not every units / skills / items
  field.
- Change (convert): `emberwake_content/tests/checks_golden.rs` with the case inputs deduplicated to one case per (kind,
  field, message) and their engine messages in a fixture (aim under 500 KB), on a frozen copy of the data the cases
  patch (the catalog of CLN-38); drop the whole-catalog report and lint comparisons (`shipped_data_checks_clean`
  covers the shipped data). Or delete (Q-A3).
- Risk: dedup may drop a case that guarded a distinct code path; keep one per message and per field. Touches:
  `games/emberwake/crates/emberwake_content/tests/{checks_parity.rs (delete), checks_golden.rs, fixtures/}`.
  Dependencies: CLN-38. Spec pointer: `docs/specs/CLEANUP.md#CLN-40`.

### CLN-41

Journey parity. Scope A. Status: proposed. Size M. Lane logic-builder. Layer game. Depends on Q-A4.

- Where: `emberwake_world/tests/journey.rs` (241 lines), `parity/journey.sh` and `parity/godot/journey.gd` (Godot's
  seeded random walker), `parity/out/journeys.json` (13.5 MB, 4 seeds x 700 steps).
- Why: the only regression over the whole game loop (explore, dialogue, shop, quests, saves, screen views). The rest of
  the loop is covered by unit-level kit tests (`rpgkit_world/tests/*`, `emberwake_world/tests/{shop_trade,npc_services,
  rewards,history}.rs`).
- Change (convert): port the walker (pick a seeded random legal action from the views) into the test, record 2 seeds x
  300 steps on a frozen copy of the shipped data into a fixture (aim 1-2 MB: action, result and a state digest per step,
  full state every 50 steps), compare step for step; re-record only for intended changes (HISTORY row). Or delete.
- Risk: the walker must be deterministic (seeded `GodotRng`, sorted choices). Touches:
  `games/emberwake/crates/emberwake_world/tests/{journey.rs, fixtures/}`. Dependencies: CLN-38 (frozen data
  pattern). Spec pointer: `docs/specs/CLEANUP.md#CLN-41`.

### CLN-42

Switch Godot off. Scope A. Status: proposed. Size M. Lane logic-builder. Layer game + kit + tools.

- Delete: `emberwake_battle/tests/trace_parity.rs`, `emberwake_world/tests/{world_traces,data_source}.rs`,
  `emberwake_world/tests/saves.rs` (with CLN-47; else converted per Q-C1), whichever of CLN-39/40/41 the user drops,
  `games/emberwake/parity/` (5 scripts, 6 `.gd` files) and its `.gitignore` line, `tools/fc.sh` `parity` command
  (:14, :149-163) and the "match Godot" summary grep (:40; check_project.ps1:57), the header lines about parity
  (fc.sh:4, check_project.ps1:3-5), gate.yml's `GODOT_VERSION`, Godot cache, install and "Record the Godot reference"
  steps (:25, :34-56) and its header (:2-6).
- Code: `emberwake_host/src/cli.rs` `catalog=`, `$AEGIS_CATALOG` and `sim parity=` (:8-9, :62-90, :111-124);
  `rpgkit_world/src/data.rs:90-98` `Data::load` (export folder) and `rpgkit_data/src/lib.rs:636` `load_resolved` unless
  CLN-38..41 read their frozen fixture through them; the sim help lists the engine-only keys (`rpgkit_sim/src/args.rs`
  comments :9, :71, :110, :116, :430, :437, feel.rs :69, :137) unless CLN-39 did it; `emberwake_gui/src/main.rs:125`
  shot folder off `parity/out`; doc comments that name the exports (`rpgkit_content/src/source.rs:2, 7`,
  `rpgkit_data/src/lib.rs:9`, `rpgkit_world/src/data.rs:5-6`, `rpgkit_battle/src/{intent.rs:359, sim.rs:289,
  log.rs:80, metrics.rs:1415}`, `rpgkit_world/src/{explore.rs:310, shop.rs:245}`, `emberwake_world/tests/report.rs:3-4`).
- Keep: `legacy/` (untouched), branch-cleanup.yml:51 (protects the `legacy` branch), fc.sh:166 (points Godot commands
  at `legacy/tools/fc.sh`).
- Risk: the gate must stay green without `parity/out`: run it on a checkout with `parity/out` removed (the desktop main
  checkout keeps a stale 51 MB copy: delete it after landing). Touches: the files above, `.github/workflows/gate.yml`,
  `tools/fc.sh`, `check_project.ps1`, `games/emberwake/.gitignore`. Dependencies: CLN-36, CLN-37, CLN-38 and each of
  CLN-39/40/41 converted or dropped; CLN-47 for saves.rs. Spec pointer: `docs/specs/CLEANUP.md#CLN-42`.

### CLN-43

Docs and rules for the end of parity. Scope A. Status: proposed. Size S-M. Lane planner (`.claude/*`, CLAUDE.md,
decisions, HISTORY, workflow docs); design-brainstorm for PORT_NOTES and ENGINE_SPEC. Lands with CLN-42.

- CLAUDE.md: line 3 ("it is the parity reference the engine is checked against") and the "Godot parity" paragraph
  become "legacy/ is frozen for reference; no build or test reads it" plus an engine-golden rule (goldens: timeline,
  PHASEFOLD and CLN-38..41's; re-recorded only for an intended change, named in the commit and a HISTORY row; never
  edited to pass).
- Decisions: new row "Godot parity retired (user 2026-11-01)" in `docs/decisions/workflow.md` (which suites were
  converted, which deleted); mark "Port goal and the end of Godot" clause (2) fulfilled; `games/emberwake/docs/decisions/
  lore.md:5, 15` "parity world" becomes "classic reference world".
- HISTORY row: "The Godot parity suites (trace_parity, world_traces, journey, saves, data_source, store resave,
  checks_parity, sim_parity, rules parity) and the recorders in games/emberwake/parity" | retired | the engine goldens
  (list) | user 2026-11-01.
- Skills and agents: `engine-checklist` description and step 5, step 4's `trace_parity` example; `rust-conventions`
  lines 34, 41, 53; `known-issues` description and rows at lines 23, 24, 25, 27, 28, 33, 44, 58 (mark retired);
  `patch-and-seed` line 25; `task-landing` line 25 (only "re-recorded golden"); agents `logic-builder` lines 13, 22 and
  `change-reviewer` lines 3, 13; `.agents/rules/engine_workflow.md` lines 17, 26, 28, 34.
- Workflow docs: `docs/STRUCTURE.md` rows at lines 45, 49, 70; `docs/WORKFLOW.md` lines 15, 32, 34, 40;
  `docs/workflow/desktop.md` lines 15 (copy `parity/out` step), 45, 47; `docs/workflow/cloud.md` lines 58, 129;
  `docs/EFFICIENCY.md` rows 15-16 (mark retired) and a new measured row; `games/emberwake/docs/GAME_BRIEF.md:18`;
  `games/emberwake/docs/SCREENS.md:8`; `engine/README.md` (12 mentions); `engine/docs/PORT_NOTES.md` (61 mentions:
  "Parity status" :42-84 and the per-system parity notes become history; archive the file per CLN-5 or cut it to the
  port map) and `engine/docs/ENGINE_SPEC.md` (22 mentions).
- Not editable: `legacy/LEGACY.md` calls `legacy/` the parity reference; the user rule forbids edits under `legacy/`
  (Q-A6 asks whether one pointer line is allowed). Spec pointer: `docs/specs/CLEANUP.md#CLN-43`.

### CLN-44

Classic as a ruleset row. Scope B. Status: proposed. Size M. Lanes logic-builder (code) then asset-builder (the row).
Layer kit + game.

- Where: `rpgkit_rules/src/ruleset.rs:10-16` (`CLASSIC`, closed `RULESETS = [classic, timeline]`, `ROW_IDS =
  [timeline]`: "classic has no row"), :327-331 (`Ruleset::classic()` hard-codes the rules list and `melee_diagonal:
  None`), :345-353 (`of`: classic short-circuits, any id but the two refused), :357 (`of_or_classic`), :369
  (`is_classic`); hard-coded `Ruleset::classic()` as a start value in `rpgkit_battle/src/session.rs:113`,
  `rpgkit_battle/src/dojo.rs:296`, `rpgkit_sim/src/args.rs:178`, `rpgkit_host/src/lib.rs:178`,
  `emberwake_gui/src/runner.rs:109`; content check `rpgkit_content/src/check/world.rs:682-687` (`r != CLASSIC`).
- Why: decision "INPUT1-M rule choices and the classic ruleset" (user 2026-11-01): classic is one ruleset template over
  the rule library, every ruleset draws on the same rule rows. Today classic is a code constant; a third ruleset cannot
  be data (the id list is closed).
- Change: kit: rulesets are rows only (`Ruleset::of` reads `cat.rulesets`; open ids, checked against the rows); the
  default id stays `classic` (`world_ruleset` default, `ruleset.rs:412-414`); a missing row falls back to the built-in
  classic values with a load note (debug convenience, like "Saves and data edits") instead of silently
  (`of_or_classic`); `Ruleset::classic()` remains only as that fallback and in tests. Data: a `classic` row in
  `catalog/rulesets.json` (rules `[]`, `melee_diagonal: none`, today's default numbers) through `aegis content`.
- Risk: classic battles must not change: timeline golden (inert-extensions half), PHASEFOLD golden, phase and host
  tests unchanged. Q-B1 (row in each game's data vs a kit default). Touches: `kits/rpg/crates/rpgkit_rules/src/
  ruleset.rs`, `kits/rpg/schema.json` (`rulesets` ids), `kits/rpg/crates/rpgkit_content/src/check/{world,ruleset}.rs`,
  `kits/rpg/crates/rpgkit_battle/src/{session,dojo}.rs`, `kits/rpg/crates/rpgkit_sim/src/args.rs`,
  `kits/rpg/crates/rpgkit_host/src/lib.rs`, `games/emberwake/crates/emberwake_gui/src/runner.rs`, then
  `games/emberwake/data/catalog/rulesets.json` (asset lane). Dependencies: INPUT1-U (shared host and session files).
  Spec pointer: `docs/specs/CLEANUP.md#CLN-44`.

### CLN-45

Split the switches bundled under `round_intent`. Scope B. Status: proposed. Size M-L. Lane logic-builder. Layer kit.

- Where: `round_intent::on` (rpgkit_battle/src/round_intent.rs:127) and `Ruleset::stances` (ruleset.rs:380) decide
  behaviours that are not the round intent itself, each a legacy-vs-new choice:
  - Rest AP timing: `plan.rs:274, 1449`, `command.rs:537`, `round_intent_plan.rs:1598` (classic returns AP on use;
    timeline pays at round end by `rest_ap_per_block`). Option: `rest_ap` = `on_use` (classic) or `round_end`.
  - Chase: `plan.rs:285` (Support chases), `:944` (chase after a positioned move), `:1276`; `command.rs:945` (root
    stops a chase). Options: `chase` = `classic` or `every_skill`, `root_stops_chase` (decision "INPUT1-M rule
    choices": "root stops a chase only under the timeline rules").
  - Target lock: `plan.rs:932, 975` (classic inserts Switch Target; timeline targets the lock). Option:
    `switch_target_step`.
  - Stances: `ruleset.rs:378-382` `stances() = !round_intent`, read in 11 places (`plan.rs:314, 1305, 1417`,
    `ai.rs:285`, `dojo.rs:772, 935, 961, 1083, 1160, 1215`, `emberwake_gui/src/dojo.rs:640`). Option: `stances`
    (classic on).
  - Diagonal melee: `ruleset.rs:362-366` reads `maneuver` as `none` without `round_intent`: drop the coupling; the
    classic row (CLN-44) sets `none`.
  - Intents list check: `rpgkit_content/src/check/ruleset.rs:75` (custom intents need `round_intent`): keep, it is the
    intent rule itself.
- Keep as they are (they are the round intent rule): `round_intent_plan.rs:109, 172, 249, 386, 1042, 1138, 1651`,
  `session.rs:996, 1053, 1085, 1301`, `ai.rs:1526, 1552`, `intent.rs:134`, `rpgkit_host/src/lib.rs:264, 852, 935`,
  `rpgkit_sim/src/timed.rs:71`, `emberwake_gui/src/battle.rs:2289`, `skill.rs:289`.
- Change: new rule options in `REGISTER` (or row fields, Q-B2), classic row off / legacy value, timeline row on;
  each call site reads its own rule. Pure refactor: every golden unchanged.
- Risk: medium (planning code in hot files); timeline golden, PHASEFOLD golden, round_intent, round_intent_plan,
  intent_ai and picks tests guard it. Touches: `kits/rpg/crates/rpgkit_rules/src/ruleset.rs`,
  `kits/rpg/crates/rpgkit_battle/src/{plan,command,round_intent_plan,ai,dojo}.rs`, `kits/rpg/schema.json`,
  `kits/rpg/crates/rpgkit_content/src/check/ruleset.rs`, `games/emberwake/crates/emberwake_gui/src/dojo.rs`, then
  `games/emberwake/data/catalog/rulesets.json` (asset lane, if row fields). Dependencies: CLN-44, INPUT1-U, INPUT1-L
  (shared plan files). Spec pointer: `docs/specs/CLEANUP.md#CLN-45`.

### CLN-46

Screens and host read rules, not `is_classic()`. Scope B. Status: proposed. Size S. Lane logic-builder. Layer game +
kit.

- Where: `rpgkit_host/src/lib.rs:428-431` (state carries `ruleset` only beyond classic), `rpgkit_battle/src/dojo.rs:313`
  (reset rebuilds decks unless classic: really "has `deck`"), `emberwake_gui/src/dojo.rs:248-257, 611, 1039` (the
  Timeline toggle flips classic and timeline), `emberwake_gui/src/runner.rs:368` (chip dim when classic and no rules).
- Change: dojo reset reads `has(DECK)`; the host state always carries `ruleset` (agent docs and any host replay
  expectations updated) or keeps the shape by a documented default; the Dojo toggle becomes a ruleset picker over
  the catalog's rows (two today); the runner chip dims at the world's default ruleset with no switches.
- Risk: host state shape (agents, `engine/README.md` host section). Touches: `kits/rpg/crates/rpgkit_host/src/lib.rs`,
  `kits/rpg/crates/rpgkit_battle/src/dojo.rs`, `games/emberwake/crates/emberwake_gui/src/{dojo,runner}.rs`.
  Dependencies: CLN-44. Spec pointer: `docs/specs/CLEANUP.md#CLN-46`.

### CLN-47

Save migrations v1-v12. Scope C. Status: proposed. Size S. Lane logic-builder. Layer kit. Depends on Q-C1.

- Where: `rpgkit_world/src/save.rs:109-243` (`quest_entry_ok`, `godot_text` "as Godot writes a value it parsed",
  `state_text`, `step` with 12 arms), :245-266 (`migrate`); test `rpgkit_world/tests/campaign.rs:340` writes a v12 save;
  `emberwake_world/tests/saves.rs` (Godot fixtures v1-v10).
- Why: decision "Saves and data edits (user 2026-11-01)": long-term save compatibility is not a goal, core systems
  need not carry migrations for old development saves. Steps 1-2 also emulate Godot's JSON text to derive an RNG seed.
- Change: `VERSION` stays 13 (or the next bump); a save below it is refused with "save version N is no longer
  supported" (load notes and validation stay); delete `step`, `godot_text`, `state_text`, `quest_entry_ok`; keep
  `aegis_save::migrate`'s version checks (the engine keeps the hook for a future final-format migration, Q-C1).
  `campaign.rs:340` moves to version 13; saves.rs is deleted (CLN-42).
- Risk: old desktop saves stop loading (intended). Touches: `kits/rpg/crates/rpgkit_world/src/save.rs`,
  `kits/rpg/crates/rpgkit_world/tests/campaign.rs`, `games/emberwake/crates/emberwake_world/tests/saves.rs`.
  Dependencies: none (land before or with CLN-42). Spec pointer: `docs/specs/CLEANUP.md#CLN-47`.

### CLN-48

The `wave_*` fold shim. Scope C. Status: proposed. Size S. Lane logic-builder. Layer kit.

- Where: `rpgkit_battle/src/encounter.rs:388-` (`fold_legacy_wave`, `LEGACY_WAVE_KEYS`), `rpgkit_battle/src/
  session.rs:85`, `rpgkit_content/src/source.rs:61, 82-101` (`folded`, `fold_legacy_waves`), `rpgkit_world/src/
  data.rs:96`; tests `emberwake_world/tests/phasefold.rs:207-220`, `store.rs:100-114`, `data_source.rs:160, 303`.
- Why: HISTORY 2026-10-22 PHASEFOLD replaced the single wave; the shipped data has no `wave_*` key (only the executed
  seed `data/seeds/reward1.json:433-435`, removed by CLN-49) and the schema has none. After CLN-42 only Godot-era data
  had them.
- Change: delete the fold and its call sites and the legacy-row tests; the PHASEFOLD golden stays. If CLN-38..41 freeze
  a catalog from the old export, freeze it folded.
- Risk: low; PHASEFOLD golden and phase tests. Touches: `kits/rpg/crates/rpgkit_battle/src/{encounter,session}.rs`,
  `kits/rpg/crates/rpgkit_content/src/source.rs`, `kits/rpg/crates/rpgkit_world/src/data.rs`,
  `games/emberwake/crates/emberwake_world/tests/phasefold.rs`. Dependencies: CLN-42, CLN-49. Spec pointer:
  `docs/specs/CLEANUP.md#CLN-48`.

### CLN-49

Executed seeds. Scope C. Status: proposed. Size S. Lane planner (or asset-builder; a `git rm`, no data edit). Layer game.

- Where: `games/emberwake/data/seeds/` (21 files, 261 KB). The README rule (lines 6-7): "a seed leaves this folder in
  the cleanup pass after its task is merged (git history keeps it)". All 20 seeds belong to done tasks: `assetorg2`,
  `cast1_a1`, `cast1_a1_alt`, `cast1_a1_pavise_keepsake`, `cast1_a1_talon`, `material_stacks` (CAST1-A1 follow-up,
  81f2350), `rare_materials` (be221e5), `derive1_d`, `encounter2_d`, `input1_ma`, `phasefold`, `portraitfx1`,
  `props1_ashlands`, `reward1`, `ruleset1_a`, `ruleset1_a_interim`, `ruleset1a2`, `ruleset1a3`, `sprite2`,
  `war2h1_gear`.
- Change: remove them; the README's engine-era table gains each file's last commit hash ("restore with `git show`"),
  as the archived Godot-era table does.
- Risk: none (no code reads `seeds/`; checked with grep). Touches: `games/emberwake/data/seeds/*`. Dependencies: none.
  Spec pointer: `docs/specs/CLEANUP.md#CLN-49`.

### CLN-50

Manual planning mode. Scope C. Status: proposed. Size S. Lane logic-builder. Layer kit + game.

- Where: `round_intent_plan.rs:137-140` (`manual`), :248-258 (`set_manual`), :1110, :220, :1142; `ai.rs:1622`;
  `rpgkit_host/src/lib.rs:341, 726, 752-763` (`manual` request), :958 (state key `manual`); `emberwake_gui/src/
  battle.rs:2356, 3663, 8587`; tests `rpgkit_host/tests/round_intent_requests.rs:67-83, 140`,
  `rpgkit_battle/tests/{intent_ai.rs:396, round_intent_plan.rs:516-579}`.
- Why: HISTORY 2026-11-01 (INPUT1-U slice 2) replaced Manual with picks (INPUT1 IF8, IF9: "skips members with a pick
  (as it skipped Manual members)"); no screen sets it any more, so `manual` is always false in play; a member with
  picks already behaves as manual (`lib.rs:958` reports `rs.manual || !rs.picks.is_empty()`).
- Change: drop the flag, `set_manual`, the host `manual` request and the state key (or keep the key as "has picks");
  the readers use `picks_of(..).is_empty()`. `fill_rest` stays (ai.rs:2154, timed.rs:133); the host `fill` request is
  test-only (round_intent_requests.rs:71): drop it too unless agents use it (Q-C2 lists it).
- Risk: low. Touches: `kits/rpg/crates/rpgkit_battle/src/{round_intent_plan,ai}.rs`, `kits/rpg/crates/rpgkit_host/src/
  lib.rs`, `games/emberwake/crates/emberwake_gui/src/battle.rs`, the three test files. Dependencies: INPUT1-U (slice 3
  shares battle.rs and the host). Spec pointer: `docs/specs/CLEANUP.md#CLN-50`.

### CLN-51

Duplicated planning in the battle screen. Scope C. Status: proposed. Size S. Lane logic-builder. Layer kit + game.

- Where: `emberwake_gui/src/battle.rs:5244-5268` (`deferred_plan` re-implements the host's `preset` and `round_intent`
  requests on copies: `set_intent`, `set_target`, `plan_unit`), the hotkey self-target rule (`battle.rs:2562`); review
  note in commit b0c195f ("deferred_plan rebuilds the host's round_intent handling in the GUI").
- Change: one kit function (beside `round_intent_plan::preview_plan` :1622) that applies a planning request to copies,
  used by the host and the screen. Superseded if INPUT1-U slice 3 takes the review minor (then mark it done).
- Risk: low. Touches: `kits/rpg/crates/rpgkit_battle/src/round_intent_plan.rs`, `kits/rpg/crates/rpgkit_host/src/lib.rs`,
  `games/emberwake/crates/emberwake_gui/src/battle.rs`. Dependencies: INPUT1-U. Spec pointer:
  `docs/specs/CLEANUP.md#CLN-51`.

### CLN-52

Split `emberwake_gui/src/battle.rs`. Scope C. Status: proposed. Size M-L. Lane logic-builder. Layer game.

- Where: 10,148 lines (the next GUI file is 2,682); 111 tests in 16 in-file test modules from :6063 to the end (`tests`,
  `windup_tests`, `arena_rules_tests`, `status_tests`, `input1_tests`, `overlay_tests`, `tells_tests`, `slice5_tests`,
  `slice6_tests`, `hook_marks_tests`, `meter_guard_tests`, `push_forecast_tests`, `hooks_audit_tests`,
  `hold_screen_tests`, ...); the screen itself mixes the board, the strip, the popup and menus, the hero panel, the log
  and the results.
- Why: decision "Housekeeper focus" priority 2 (split files that mix systems); every battle UI task conflicts here.
- Change: step 1 (mechanical): `battle.rs` becomes `battle/mod.rs` with the test modules in `battle/tests/*.rs`; step 2:
  board, strip, popup, hero panel as submodules. No behaviour change.
- Risk: merge conflicts with in-flight battle UI work, so only at a flush point. Touches:
  `games/emberwake/crates/emberwake_gui/src/battle.rs` (to `battle/`). Dependencies: INPUT1-U, INPUT1-T, INPUT1-PB,
  INPUT1-AP, VISCLAR1-A landed. Spec pointer: `docs/specs/CLEANUP.md#CLN-52`.

### CLN-53

Oversized docs. Scope C. Status: proposed. Size S-M. Lane design-brainstorm. Layer docs.

- `tools/fc.sh docs` warns on 36 files today, so the warning no longer signals anything. The named ones:
  - Built specs: `games/emberwake/docs/specs/STAGE1.md` (364 lines, 29 KB; Status "done; reviewed" 2026-10-27) and
    `WINDUP1.md` (401 lines, 34 KB; WINDUP1-R2 and -R3 done 2026-10-29): archive per CLN-5 (first line, archive
    README row), like CLN-30's built Godot-era specs.
  - Live lore: `games/emberwake/docs/lore/CANON.md` (333 lines, 39 KB) and `STORY_GUIDE.md` (209 lines, 26 KB):
    split by domain behind an index (CANON by section: world, factions, cast, acts; STORY_GUIDE by act); the
    Worldbook feedback is pending (memory note), so after that review.
  - Live specs with open rows: `docs/specs/INPUT1.md` (785 lines, 72 KB; INPUT1-U, -T, -L, -PB, -AP open),
    `ROSTER1.md` (410 lines; 3 rows approved), `CAMPAIGN1.md` (384 lines; CAMPAIGN1-TM2, -U open): split into an index
    and per-section files once their running rows land (splitting under running builders breaks their section
    pointers); their Status lines still read `proposed` (CLN-23 already covers that).
- Rule proposal: archived and `done` specs leave the warning list (the check skips `docs/archive/`), so the warning
  stays meaningful. Touches: the docs above, `docs/archive/README.md`. Dependencies: CLN-30 (same archive pattern).
  Spec pointer: `docs/specs/CLEANUP.md#CLN-53`.

### Pass 3 questions

- Q-A1 (CLN-42): delete `trace_parity` and `world_traces` outright (recommended: the PHASEFOLD golden, the timeline
  golden and the kit tests cover classic battles, world rosters and phases), or keep a slim engine golden of the
  battle-item (pouch) fights and the synthetic world variants (escape, centre deployment, hold tiles, start buffs),
  about 1 MB?
- Q-A2 (CLN-39): sim output golden on a frozen catalog (recommended, S) or delete?
- Q-A3 (CLN-40): content checks: slim golden of the broken-entry cases (recommended, M: the only broad check of the
  old kinds' messages) or delete?
- Q-A4 (CLN-41): journeys: an engine random-walk golden (M; the only whole-loop regression) or delete (the kit tests
  stay)? Recommendation: convert, the game loop has no other end-to-end check.
- Q-A5 (CLN-36): delete `timeout_verdict.rs` (recommended, a one-off move check) or point it at the shipped catalog?
- Q-A6 (CLN-38, CLN-43): may the goldens share one frozen fixture copied from the last Godot export (catalog, worlds,
  game data, ~290 KB, folded) or must they freeze the shipped data (recommended: shipped data, no Godot-made file
  stays)? And may `legacy/LEGACY.md` get one line saying no build reads it (the rule says never edit `legacy/`)?
- Q-B1 (CLN-44): the `classic` row in each game's `catalog/rulesets.json` (recommended; the decision calls classic a
  template) or a kit default?
- Q-B2 (CLN-45): the split switches as rule options in the register (on/off per ruleset, like `deck`) or as row
  fields with values (`rest_ap: on_use | round_end`)? Recommendation: row fields where there are more than two
  choices, options otherwise.
- Q-C1 (CLN-47): refuse every save below the current version (recommended), or keep the last one or two migrations?
- Q-C2: rename the Godot-named identifiers (`GodotRng` 59 uses in 16 files, `godot_sort`, `godot_list`)? Recommended
  no: churn without behaviour value. Drop the test-only host `fill` request with CLN-50?

### Pass 3 checked and kept

`legacy/` and the `legacy` branch (user rule); `timeline_golden.rs` and both fixtures, `phasefold.rs` and its golden
(engine-owned); `emberwake_sim/tests/rules.rs`, `emberwake_world/tests/intents.rs`, `report.rs` (no Godot input; only
comments mention it); `aegis_core` Godot-semantics code (`GodotRng`, `ghash`, `json::normalize`): the engine's numeric
behaviour; `round_intent::on` reads that are the round intent rule itself (listed under CLN-45 "Keep");
`fill_rest` (AI and sim use it); Ruleset `RuleKind` Option vs Extension (still meaningful once classic is a row).

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
