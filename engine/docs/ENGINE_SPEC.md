# Aegis Engine Spec (agent-native engine, parallel to Godot)

Status: draft, branch `engine` only (2026-10-18). Not merged to `main`; not in
`docs/ROADMAP.md` or `docs/DECISIONS.md` until the user approves it.
Supersedes the Markdown-as-language direction in `../ENGINE_PROPOSAL.md`.

## 1. Goal

Build a custom Rust engine for the tactical RPG that runs **in parallel** with
the Godot build and reaches **feature parity** with it, system by system. The
Godot game on `main` is the reference (the oracle) and stays the shipping build
until the user decides otherwise.

The engine exists for three reasons (user, 2026-10-18):

1. **Full-stack visibility.** Agents can read, change, build and test every
   layer, including the engine itself. No closed or out-of-reach layer where a
   bug can only be worked around.
2. **Built for agents.** Agents develop, debug and maintain it. Human
   ergonomics (editor, inspector, visual scene authoring) are not goals.
3. **Close the feel gap.** The user cares about look and game feel, which
   agents cannot experience. The engine must turn feel into data the agent can
   read and change, and make user feedback precise.

## 2. Principles

| # | Principle | What it means in practice |
| :--- | :--- | :--- |
| P1 | Text only | No binary scenes, no editor-only state. Everything is code, data or a text spec. |
| P2 | Headless first | Every system runs and is tested without a window. The GUI is an optional client. |
| P3 | Deterministic | Same seed plus same command log gives the same state, every time, on every machine. |
| P4 | One command path | GUI clicks and CLI calls go through the same command API. No GUI-only logic. |
| P5 | Introspectable | Any state, and the current UI tree, can be dumped as JSON on demand. |
| P6 | Mainstream code | Idiomatic Rust, strong types, small modules. No custom language. Readability for humans is not required, but idiomatic code is also what agents get right most often. |
| P7 | Specs readable by the user | Rules, numbers and formulas stay in tables the user can check, even if engine code is not for humans. |
| P8 | Feel is data | Every timing, curve, shake, flash, colour and volume is a named value in a tuning file, never hard-coded. |
| P9 | Small and specific | Build only what this game needs. Not a general-purpose engine. |
| P10 | Fast loop | Edit, build, test in seconds. Data and tuning hot-reload without a rebuild. |

## 3. Boundaries

- All engine code, data, specs and tools live under `engine/`. Nothing outside
  `engine/` changes for engine work.
- `engine/.gdignore` keeps Godot from scanning the folder (including the
  multi-GB `target/`).
- The engine **reads** shipped content from the Godot project (`data/`, world
  files) read-only. It never writes there. Engine-only data lives in
  `engine/data/`.
- The branch `engine` is not merged to `main`. It is rebased on `main` when
  `main` moves.

## 4. Architecture

```text
            ┌──────────────── aegis game process ───────────────┐
 User ───►  │  GUI client (egui/wgpu): draws state, sends cmds  │
 (mouse)    │        │                                          │
            │        ▼                                          │
 Agent ──►  │  Command API ──► Sim core ──► State ──► Event log │
 (CLI)      │        ▲            (deterministic)     (replay)  │
            │        └──── inspect: state / UI tree / tuning    │
            └───────────────────────────────────────────────────┘
```

### 4.1 Crates

| Crate | Role | Depends on | Window |
| :--- | :--- | :--- | :--- |
| `aegis_core` | `IVec2`, seeded RNG, ids, small shared types | none | no |
| `aegis_data` | Loads Godot `data/` JSON (read-only) and `engine/data/`, typed rows, validation | core | no |
| `aegis_rules` | Pure rule functions: damage, armor, crit, stance matchup, Focus, statuses, terrain, combos | core, data | no |
| `aegis_sim` | Battle and game state machines; applies commands; emits events | rules | no |
| `aegis_proto` | Command, query and event types (serde JSON); record/replay format | sim | no |
| `aegis_host` | Runs a session and serves the protocol over stdin/stdout and a local socket | proto | no |
| `aegis_cli` | Agent client: run, step, inspect, replay, diff, golden checks | proto | no |
| `aegis_tune` | Tuning file schema, hot reload, variant sets | core | no |
| `aegis_gui` | Thin view: renders state and UI tree, maps input to commands, tuning panel | proto, tune | yes |
| `aegis_parity` | Golden-trace runner: compares sim output with traces from Godot | sim | no |

Rules stay pure functions with no I/O, so they are cheap to test and to diff
against Godot. The sim never imports the GUI. The GUI never decides a rule.

### 4.2 Protocol (outline; full schema is task E2)

Line-delimited JSON over stdin/stdout (agent runs a session) or a local socket
(agent attaches to the game the user is playing).

| Kind | Examples |
| :--- | :--- |
| Command | `select_unit`, `move`, `use_skill {skill, target}`, `end_turn`, `set_stance`, `autoplay_round` |
| Query | `state` (full or a path), `ui_tree`, `legal_actions {unit}`, `forecast {unit, skill, target}`, `tuning` |
| Event | `damage_dealt`, `status_applied`, `unit_defeated`, `turn_started`, `battle_ended` |
| Session | `new {seed, encounter}`, `load {save}`, `record`, `replay {log, until_step}`, `screenshot` |

Every reply carries a `step` counter. A command log plus the seed reproduces
the session exactly (P3).

### 4.3 UI tree

The GUI publishes what is on screen as a tree: node id, kind (panel, button,
label, grid cell, unit sprite), text, enabled or disabled, highlighted,
screen rect. `ui_tree` returns it as JSON. This is how an agent "sees" what the
user sees. Screenshots are for visual bugs only.

## 5. Feel tooling (closing the feel gap)

| Tool | What it does | What the agent gets |
| :--- | :--- | :--- |
| Tuning file | `engine/data/tuning.json`: every feel value by name (hit pause ms, shake px, ease curve, flash colour, damage-number rise, SFX volume) | Exact, diffable values |
| Live tuning panel | In-game sliders; changes save to the tuning file | The user's change as a diff ("hit pause 40 → 70 ms") |
| "Feels off" hotkey | Captures screenshot, last N seconds of command log, state and a typed note into `engine/feedback/<timestamp>/` | The exact moment, replayable headlessly |
| Variant switcher | Agent ships 2-3 named tuning sets; user flips between them in game and picks one | A clear choice instead of a vague description |
| Feel metrics | Input-to-feedback latency, animation durations, "every action has a visible response" checks | Measurable rules the agent can test |
| Feel vocabulary | User words ("snappy", "heavy") mapped to parameter sets, kept in `engine/docs/FEEL.md` | Shared language that improves over time |

## 6. Parity with Godot

Parity is measured on **behaviour**, not pixels.

1. **Golden traces.** A seeded battle in Godot writes a JSONL trace: every
   command, roll and resulting event with numbers. The same seed and commands
   in `aegis_sim` must give an identical trace. `aegis_parity` diffs them and
   shows the first divergence.
2. **Rule unit parity.** Each pure rule (for example damage `compute()`) gets a
   table of inputs and Godot-computed outputs; the Rust function must match.
3. **RNG parity.** Exact trace parity needs the same random stream. Options:
   port Godot's PCG32 bit-for-bit, or make the trace record every roll and feed
   them back. Decision needed (task E1).
4. **System checklist.** `engine/docs/PARITY.md` lists Godot systems
   (combat core, stances, Focus, statuses, terrain, combos, AI, intents,
   autoplay, exploration, dialogue, camp, saves) with status:
   `not-started / rules / sim / gui / parity`.

The Godot trace exporter is a GDScript tool. To keep everything under
`engine/`, it lives in `engine/parity/godot/` and is run with
`godot --headless --path .. --script <abs path>`. Whether a script under a
`.gdignore` folder can load project classes that way must be checked first
(task E1); if it cannot, the user decides where the exporter goes.

## 7. Current branch: keep, rework, retire

| Item | Verdict | Reason |
| :--- | :--- | :--- |
| `aegis_core` (IVec2, RNG, Signal, Variant) | Keep `IVec2`, RNG; drop `Variant` and `Signal` later | Typed state and events replace dynamic values and signals |
| `aegis_vfs` catalog loading | Rework into `aegis_data` | Typed rows instead of generic JSON |
| `aegis_md_parser`, `aegis_gd_bridge`, `game_specs/` | Retire (keep in git history) | Markdown-as-language dropped (P6). The parser does not parse logic, and the bridge turned method locals into class properties |
| `aegis_vm` `Evaluator`, `Scope` | Retire | No DSL to evaluate |
| `aegis_vm` `combat.rs` | Retire | Hand-written placeholder rules (`attack - armor`, first-alive targeting) that do not match the game |
| `aegis_app` | Rework into `aegis_gui` | Holds a separate combat copy (own `Unit`, `Stance`, damage). Keep the drawing code, move all rules into `aegis_rules` / `aegis_sim` |
| `web/` player | Park | A further copy of combat in JS; revisit once the protocol exists (could become a protocol client) |
| `aegis_sim`, `aegis_game`, `aegis_bench` | Rework into `aegis_host` / `aegis_cli` | Wrap the old placeholder combat |

There are four combat implementations today (Godot, `aegis_vm`, `aegis_app`,
`web/`). The target is one in Rust (`aegis_rules` + `aegis_sim`), checked
against Godot.

## 8. Milestones

| Id | Deliverable | Done when |
| :--- | :--- | :--- |
| E0 | This spec approved; `.gdignore`; retire list agreed | User sign-off |
| E1 | Trace format, Godot trace exporter, RNG parity decision | One Godot battle exports a trace |
| E2 | `aegis_proto` schema; `aegis_host` + `aegis_cli` skeleton | Agent can `new`, `state`, `end_turn`, `replay` headlessly |
| E3 | `aegis_rules` damage path (attack power, armor, crit, stance, guard) | Rule-table parity with Godot `DamageCalculator` |
| E4 | Combat sim core (turns, Focus, skills, statuses, AI) | Golden-trace parity on the core encounter set |
| E5 | `aegis_gui` on the protocol, UI tree, tuning file | User plays a battle; agent reads `ui_tree` of the same screen |
| E6 | Feel tooling (panel, hotkey capture, variants, metrics) | One feel issue fixed end to end through the tools |
| E7 | Remaining systems by checklist | `PARITY.md` all `parity` |

Godot keeps shipping during this. Each milestone re-runs parity against the
latest `main` to catch drift.

## 9. Open questions for the user

1. RNG parity: port Godot's generator exactly, or record and replay rolls? (E1)
2. Where the Godot trace exporter may live if it cannot run from `engine/`.
3. GUI stack: stay on egui (fast to build, less game feel) or move to a custom
   wgpu 2D renderer for the battle view with egui only for panels?
4. When to formally add the engine to `docs/ROADMAP.md` and `docs/DECISIONS.md`.
