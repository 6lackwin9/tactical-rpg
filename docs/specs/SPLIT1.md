# SPLIT1: a game-agnostic engine and the first game, Emberwake

Status: approved (user 2026-10-19, "all recs": sections 2-6 as written, SPLIT1 before the PORT rows, saves copied into the per-game folder). Planner, 2026-10-19. Inputs: the user's direction of 2026-10-19 (the engine is a separate, reusable entity; changes land in the
approves. Inputs: the user's direction of 2026-10-19 (the engine is a separate, reusable entity; changes land in the
layer they belong to; games reuse each other's content by identity only; one rule set per game; monorepo for now;
the current game is `emberwake`, its title kept as metadata).

## 1. Principles

1. **Layer discipline.** Every change first asks which layer it belongs to and lands there. The engine is never
   "finished"; it grows when a game needs something that is not game-specific.
2. **Extract, do not invent.** The engine core takes what the current game shows to be generic. Something that only
   one game uses stays in that game until a second user appears (the kit layer is built the same way).
3. **Move only.** The refactor changes where code lives, never what it does: every stage keeps the gate and the Godot
   parity suites green. Clean-ups found on the way become their own tasks.
4. **Parity is a benchmark** (rewording of decision "Godot parity as a migration check"): the parity suites measure
   what the engine does against the Godot build and show what is missing, so the design work done in Godot is
   reused without depending on Godot; an intended design change simply retires the checks it affects.

## 2. Layers

| Layer | Holds | Knows about |
| --- | --- | --- |
| Engine core (`engine/`) | app shell and window, 2D drawing and UI toolkit, assets (images, strips, audio), the schema-driven content layer (store, loader, writer, validation hooks, file format), the world graph (worlds, inheritance, connects, qualified ids, overrides, cross-game references), the save framework (envelope, versions, migration runner, slots), deterministic RNG and hashing, the command API protocol (requests, history, replay), the agent link, record and replay, dev-tool shells (content viewer, scene runner) | nothing about any game's rules or content |
| Kit (`kits/`, later) | rule options a type of game chooses from; extracted from a game when a second game shares them | engine core |
| Game (`games/<id>/`) | one rule set, its content and assets, story, screens and flow, its schema (kinds and fields), its parity tools and docs | engine core (and later one kit) |

## 3. Module map (what moves where)

| Today | Engine core | Game (Emberwake) |
| --- | --- | --- |
| `aegis_rules` | `rng.rs`, `ghash.rs`, `sort.rs` -> `aegis_core` | stats, gear, unit building, damage, statuses -> `emberwake_rules` |
| `aegis_data` | `value.rs` (lenient field readers) -> `aegis_core` | typed rows (classes, units, skills, items, passives) -> `emberwake_data` |
| `aegis_world::json` | Variant helpers -> `aegis_core` | - |
| `aegis_world::source` / `store` | normalize and coerce by schema, stringify (file format), file and folder layout, upsert / remove / commit / check with hooks -> `aegis_content` | skill ranks and SkillText, item families and modifiers, the kinds' storage rules, Forge checks -> `emberwake_content` (hooks) |
| `aegis_world::data` | content store, world graph (lineage, merged, resolve, qid) -> `aegis_content` | character units, map / quest / service lookups -> `emberwake_world` |
| `aegis_world::save` | envelope, version, migration runner, slot files -> `aegis_save` | GameState shape and its migrations (v1 -> v10) -> `emberwake_world` |
| `aegis_world` (rest) | - | game state, new game, progression, exploration, quests, dialogue, shops, travel, purse, battle bridge -> `emberwake_world` |
| `aegis_battle` | - | the whole battle system -> `emberwake_battle` |
| `aegis_host` | request routing, history, replay protocol -> `aegis_host` | the battle command set -> `emberwake_host` |
| `aegis_gui` | `canvas`, `painter`, `ui`, `art` (loader), `tuning`, `bridge` (agent link), the screen stack and notices of `screens`, the app loop of `main` -> `aegis_ui` | every screen (`menus`, `region`, `worldmap`, `hub`, `party`, `battle`, `forge`, `dojo`, `runner`) and the game context -> `emberwake_gui`; binary `emberwake` |
| `aegis_cli` | the shell and the generic commands (`content`, `link`, `serve`) -> `aegis_cli` (game registered at start) | `sim`, `battle` -> registered by Emberwake |
| `engine/data/schema.json`, `tuning.json` | - | `games/emberwake/schema.json`, `games/emberwake/tuning.json` (the engine reads a game's schema; it owns none) |
| `engine/parity/`, parity tests | - | `games/emberwake/parity/` and the game crates' tests (they benchmark Emberwake against the Godot build) |

The game context (`screens.rs` `Cx`: data, game state, saves, settings, art, log) splits into an engine context
(settings, art, tuning, notices, navigation) and a game context the game owns.

## 4. Repository layout

```text
Cargo.toml                 one workspace: engine/crates/*, games/*/crates/*
engine/
  crates/                  aegis_core, aegis_content, aegis_save, aegis_ui, aegis_host, aegis_cli
  assets/fonts/            the UI font
  docs/                    engine docs (ENGINE_SPEC, PORT_NOTES while porting, engine design notes)
games/
  emberwake/
    game.json              the game manifest (section 5)
    schema.json tuning.json
    crates/                emberwake_rules, emberwake_data, emberwake_content, emberwake_battle,
                           emberwake_world, emberwake_host, emberwake_gui (binary `emberwake`)
    data/ assets/          content and art (today's root data/ and assets/)
    parity/                the Godot benchmark tools and recordings
    docs/                  game design: brief, roadmap, game decisions, specs, balance, preferences, screens
kits/                      later
docs/                      studio process: workflow, efficiency, agent map, process decisions, history, task queue
legacy/  tools/  .claude/  .agents/  .github/  CLAUDE.md  run.bat  check_project.ps1
```

One task queue stays at `docs/TASKS.md` with a new `Layer` column (engine, kit, game), so one planner sees the
whole picture.

## 5. Game manifest and title

`games/emberwake/game.json` (read by the engine at start; stage 8 adds it to `aegis content`, so it is never hand-edited):

```json
{"id": "emberwake", "title": "Emberwake", "subtitle": "", "version": "0.1.0", "kit": "", "engine": ">=0.1",
 "worlds": ["ashlands", "iron_reaches", "tidewrack"], "start_worlds": ["ashlands"], "references": []}
```

- `id` is stable: folder, crate prefix, save folder (`%APPDATA%/Aegis/emberwake/`), qualified ids and references
  use it; a retitle never changes it.
- `title` and `subtitle` are display metadata: window title, main menu, credits, save slot headers read them; no file
  hard-codes the game's name (today: `main.rs` "Aegis - Tactical RPG", `menus.rs` "Tactical RPG").
- Text that names the game uses the token `{game.title}` (resolved when text is shown), so a retitle is a one-line
  manifest change; lore written around the name itself still needs a writing pass.

## 6. Cross-game references (format reserved now, built when a second game exists)

- Qualified ids gain the game: `<game>:<world>:<id>` (inside one game `<world>:<id>` stays valid).
- `game.json` `references`: `[{"game": "<id>", "version": "<semver range>"}]` pins what a game reuses.
- Identity only: the schema marks identity fields (`"identity": true`: name, description and flavour, art, lore,
  relationships, map layout); a referencing entry is `{"id": ..., "from": "<game>:<world>:<id>", <mechanics>}`. The
  loader copies the identity fields from the source; every mechanics field (power, multipliers, statuses, rules) comes
  from the referencing game, and `aegis content check` reports the missing ones.
- When the source game's version moves past the pinned range, a drift report lists changed identity fields. Agents
  fill gaps and raise conflicts to the user; the user's answer decides.
- The three current worlds stay one game for now (only Ashlands is fleshed out); they are the first test of
  references once built.

## 7. Staged plan (each stage one task, move only, gate and parity green, landed on `develop`)

| Stage | Task | Size |
| --- | --- | --- |
| 1 | Workspace at the repo root (`Cargo.toml`, toolchain and target config), paths in tools, CI and `run.bat` | S |
| 2 | `aegis_core`: RNG, hash, sort, Variant helpers, field readers | S |
| 3 | `aegis_content`: schema-driven loader and writer, file format, world graph, with game hooks for skill ranks and item families | L |
| 4 | `aegis_save`: envelope, versions, migration runner, slots; the game keeps its state shape and migrations | M |
| 5 | `aegis_ui`: toolkit, art loader, tuning, agent link, screen stack, app loop; the engine and game contexts split | L |
| 6 | `aegis_host` and `aegis_cli`: generic protocol and shell; Emberwake registers its commands | M |
| 7 | Game crates renamed and moved to `games/emberwake/crates/` (rules, data, content, battle, world, host, gui) | M |
| 8 | `games/emberwake/`: data, assets, schema, tuning, parity tools; `game.json` with the title read everywhere; saves under the game id (the existing `%APPDATA%/Aegis/saves` copied, not moved) | M |
| 9 | Process: the `Layer` column in TASKS, layer check in the change-reviewer, STRUCTURE, CLAUDE.md, game docs into `games/emberwake/docs/` | S |

Order: 1 then 2 (everything uses them), 3 and 4, 5 and 6, then 7 and 8 (the big moves, once the boundaries exist),
then 9. Each stage is reviewable on its own; parity proves behaviour did not change.

## 8. The PORT rows by layer

| Row | Layer | Note |
| --- | --- | --- |
| PORT-INTENT | game | battle rule plus its screen |
| PORT-BREPORT | game | metrics and report are Emberwake rules |
| PORT-SIM | game | the balance sim of Emberwake's rules |
| PORT-AUDIO | engine (playback, buses, crossfade) + game (which sound and music when) | |
| PORT-SETTINGS | engine (settings store, text size, display, volume) + game (its own keys) | |
| PORT-FOCUS | engine | UI toolkit |
| PORT-PLAYTEST | engine (record and replay over the agent link) + game (what a session records) | |
| PORT-CHECKS | game (the rules) on engine check hooks | |

Recommendation: run SPLIT1 before the PORT rows (the engine-core ports then land in the right crate from the start);
PORT-INTENT and PORT-BREPORT could run in parallel with stages 2-6, accepting a rename in stage 7.

## 9. Out of scope

The kit layer itself, cross-game references beyond the reserved format, a separate engine repository, a web build,
renaming the `aegis` CLI, any behaviour change.

## 10. Questions for the user

1. Approve the layers, module map and layout (sections 2-4)?
2. Approve the manifest and title rule (section 5) and the reserved reference format (section 6)?
3. Run SPLIT1 before the PORT rows (recommended), or interleave PORT-INTENT and PORT-BREPORT?
4. Saves: copy today's `%APPDATA%/Aegis/saves` into the per-game folder in stage 8 (recommended), or start fresh?
