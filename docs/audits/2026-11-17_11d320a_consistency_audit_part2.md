# Full Project Review — 2026-11-17 (`@11d320a`), part 2

Continues [part 1](2026-11-17_11d320a_consistency_audit.md) (summary, automation path, code, lore and workflow).

## Art and UI

Nothing is missing in the sense of a broken reference: every Act 1 unit, skill and item resolves to a file. The problem is that the game ships four unrelated art styles and no locked one. The most visible clash is grim, well-painted portraits beside bright, flat, generic-fantasy battle sprites. I looked at the founder portraits, the unit tokens and the skill icons myself and agree with this verdict; the contact sheets are attached to the reply.

**Quality by category:**

| Category | State | Verdict |
| --- | --- | --- |
| Founder portraits (6, plus 48 faces) | Strongest set: one hand, consistent framing and light | Off canon. They show late-stage corruption (Kaelen's bone spurs, Hollis's red eye, Dunstan's jaw graft) against the "clean Day 1" lock. Elyse's grin and teal read least grim. Re-render is already requested (AI-04). |
| NPC and boss portraits (about 50) | Two looks: 14 newer citizen portraits are desaturated and fit best; the older framed set is warmer | Vane, Quell and the Purifier are the best. Orlov reads as a different game. Six cohort recruit busts and three boss portraits are placeholders. |
| Battle sprites (35 sheets) | Fully animated, 35 to 38 clips each, from the procedural rig | Weakest link. Flat shading, heraldic red and blue, a wizard hat, a white priest. Every named character is a class body with a palette swap, so Kaelen's token looks nothing like his portrait. Orlov and the Amalgam use the brawler body. |
| Skill, item and status icons (about 310) | Three styles in one menu: painted on red, clean slate tiles, flat glyphs | 89 skill icons and about 50 item icons are deterministic placeholders. The slate tiles are the most readable at small size. |
| Tiles and props | Props are good and coherent (about 90 of 102) | Five ground tiles are near-flat dark fills; pixel density is mixed (256, 128 and 32 px sources). |
| Backgrounds (38) | The four VN stages, the M5 gate, the title and the act maps are release quality | The opening vignette (intro 1 to 4), hearth, meat market and all 15 battle backdrops are procedural and look two generations older. |
| Music and SFX | Menu, town and three combat tracks are final | Eight synth placeholders; all SFX are generic. Not listened to, status is from the READMEs. |

**Acts 2 and 3 need art that does not exist yet:** six boss portraits, final recruit busts, oversize sprites for the two big bosses, and tiles, props, backdrops and music for both new regions.

**UI.** All 12 screens are built in code with the Tallow and Iron theme, but every committed screenshot is from 10-18 to 10-20 and shows the old navy debug look. There is no current capture, so nobody, including an agent, can review the UI from the repo. Screens with no layout at all: story card banners, endings, credits, and a loading or first-run screen for the web build. The battle checklist in NEEDS\_YOUR\_EYES is entirely unticked.

**Approaches used so far:**

| Approach | What it produced | Keep, change or drop |
| --- | --- | --- |
| Procedural sprite rig (rig.py, now `tools/sprites`; Brawler signed off 10-18) | 35 complete, deterministic sheets | Keep the rig and animations. Change the paint: grim palette ramps, three-tone shading, per-character heads and outfit parts. |
| Procedural art (`gen_art.py`) | Backdrops, title cards, old icons | Keep for placeholders only, and label them. |
| Image generation plus pixelize (portraits) | Founders, bosses: the most consistent set in the repo | Keep as the portrait finisher. Each run needs your approval. |
| Local FLUX / ComfyUI | Drafts, tiles | Desktop only. Do not plan cloud work on it. |
| Flow characters, video-to-sprite, Flow Music | Kaelen's stage set, CGs, five tracks: highest quality | Keep for CGs, VN bodies and music. Drop per-character video animation; only Kaelen ever got it. |
| Deterministic placeholder packs | About 150 icons, busts and props that unblocked Build | Keep, but track them. The replacement backlog lives only in prose. |
| Mock UI (`tools/mock_ui`) | Playable HTML flow, fast layout decisions | Change: layout studies only. It is a second implementation that drifts from the real UI; review from engine captures instead. |
| Request inbox and asset plan | Seven open requests, an 11-batch plan | Merge. Status is spread over five documents and the outbox is empty. |
|  |  |  |

**Improved flow.** This keeps your 10-18 preference (one heavy live session to lock the style, then background batches with one final look) and adds the two things it lacked: a written style lock and a single tracker.

1. **One style-lock session with you.** One board: the founder portraits, a repainted Kaelen token and Watch footman, six icons, one ground tile, one battle scene. You decide portrait frame and saturation, the sprite palette and the icon template. The result is a one-page style bible with a palette file that every later batch is checked against.
2. **One asset status table** (path, category, final / placeholder / missing, source, act), generated from the folders, replacing the five places status lives now.
3. **Background batches of 10 to 30 same-type assets.** Each returns one contact sheet at game size and at 4x, before and after, on the real background, with a five-point check: identity, palette, pixel grid, readable at size, canon.
4. **You look once per batch.** Rejects go back with one note; replacements land in place.
5. **Full UI review from fresh engine captures** once the Act 2 and 3 features land. The mock-up picks stay preferences until then.

**Top Act 1 jobs in order:** (1) repaint rig palettes and heads for the six founders and the Watch; (2) founders' Day 1 portraits; (3) opening vignette, hearth and meat market in the VN-stage style; (4) one icon template across skills, items and statuses; (5) the five placeholder boss and NPC portraits; (6) seven battle backdrops; (7) the five flat ground tiles; (8) boss, danger and travel music plus stingers.

## Engine track: Aegis as its own product

Aegis today is a strong UI and tooling engine, not yet a general 2D engine. `aegis_ui` is 75% of the engine code (40.6k of 54k lines), there is no world layer (camera, tilemap, entities, pathfinding), and nothing proves a second game can be built on it. Its standout strength is exactly what automation needs: an agent can already drive and inspect the game with no window.

**Maturity by subsystem:**

| Subsystem | Maturity | Note |
| --- | --- | --- |
| UI toolkit, theme, widgets | High | Focus, drag-drop, windows, rich text, 51-role theme, gallery |
| Assets, VFS, bundling | High | Three-layer lookup, pak parts |
| Save and store | High | Envelope, migration runner, web store |
| RNG and determinism | High | Golden-tested; still carries Godot-compatible semantics |
| Agent tooling | High | Headless shell, agent link (click, type, screenshot, UI tree, audits), scenario runner, deterministic replay |
| Web / WASM | Medium-high | Shipped, with a browser smoke test that is advisory only |
| Rendering | Medium, narrow | CPU software canvas. No rotation, shaders, batching or tilemap layer |
| Audio | Medium | Four buses, music fade, loop points. No ducking, pan or pitch |
| Animation | Medium | Clip player and fixed effects; no tween library, no particle emitters |
| Input | Low-medium | Keyboard and mouse only. No gamepad, no rebinding, touch is a stub |
| Performance tooling | Low | No benchmarks; the scenario perf step is a stub |
| Platforms | Windows first | CI is Linux only and runs on `main`, not `develop`. macOS never built |

**Absent for a second game:** a stable API or versioning, a `new game` scaffold, an author's guide to writing a screen (the engine's authoring docs are mostly RPG method and name Emberwake), localisation, gamepad, pathfinding outside the kit, networking, property or fuzz tests. There is no engine binary: the `aegis` command is built from the game's host crate.

**Quality.** Low panic density (67 unwrap or panic sites, 48 in one file) and one `unsafe` block, which is good. The weak points: `shell.rs` is 4,311 lines mixing window, input, link and settings; RPG nouns remain in the engine (recordings labelled "BATTLE", trail flags with unit and round, a hardcoded "arena" screen in the scenario runner, portrait effects and pips widgets); clippy is advisory with no lint config; and only two scenarios exist, which the gate parses but never runs.

**Recommended engine track**, ranked by value. All are safe to build unattended unless marked.

| # | Task | Size | Helps |
| --- | --- | --- | --- |
| 1 | Scenario image goldens: a shot step that compares against a committed baseline, with all scenarios run headless in the gate. Baseline changes need your approval. | M | Automation: catches visual regressions with no human |
| 2 | Kit-less sample game `games/hello/` (menu, moving sprite, save, sound, one scenario) built and run in the gate | M | Proves the engine is game-agnostic; flushes out RPG leaks |
| 3 | Replay in CI: the inbox recordings as regressions, plus a seeded UI bot that presses valid widgets and asserts no panic | M | Automation: gameplay regressions |
| 4 | Real perf step with a headless frame-time budget, plus benchmarks for fill and text | M | Automation; both games |
| 5 | Split `shell.rs` into input, link, settings and window modules, behaviour-identical | M | Cheaper and safer agent edits |
| 6 | CI hardening: a Windows job (the shipping target has none), a small deny-lint set, web smoke required, unused dependencies removed | S | Automation |
| 7 | Engine purity: caller-supplied labels for recordings, trail and scenario launch; move portrait effects, pips and vignette to the kit. Needs a design call on the moves. | M | Second game |
| 8 | `aegis new game` scaffold and an engine API guide; move the RPG authoring pages to the kit | S-M | Future games |
| 9 | Input layer: action map with rebinding and gamepad. Feel needs a playtest. | M | Both games, accessibility |
| 10 | Kit UI crate: move the generic RPG screens out of `emberwake_gui` (see Code and specs). Needs a spec first. | L | Second RPG; removes the biggest layer debt |
| 11 | World primitives: camera, tilemap layer, grid pathfinding lifted from the kit. Needs a spec first. | L | Future games |
| 12 | Localisation string tables; crash and log file; property tests for layout, pak and save migration | S-M | Robustness |

Items 1, 3, 4 and 6 are also the guardrails the automation plan depends on, so the engine track and the automation plan start with the same work.

## Recommended next steps

All of these are proposals. Nothing in the repo was changed by this review.

| # | Step | Who | Size |
| --- | --- | --- | --- |
| A | **Instruction cleanup.** Fix the workflow and skill contradictions, correct the six READY rows, mark the accepted B1 to B7 rulings, define playtest intake, one shippable-milestone definition and the backlog order. Draft the autopilot mode as a decision for you. | Design | One pass, docs only |
| B | **Small fix slices.** Player-visible placeholder and "gold" text, the M5 quest name, and the four spec mismatches N1 to N4. | Design queues, Build builds | 2 to 3 slices |
| C | **Guardrail slices.** Scenario image goldens, replay and balance-band checks, CI on `develop`, automatic audit with a recorded commit, balance agent on `develop`. | Design queues, Build builds | 4 to 6 slices |
| D | **Art style-lock session.** Open the Art/UI thread with the one-board review, then the status table and background batches. | Art/UI with you | One live session |
| E | **Archive this review** as the second report in `docs/audits/` so the next audit can diff against it. | Design | Small |
| F | **Lore rulings.** Design brings you the seven lore questions as one card set with a recommended option each. | Design, then you | One round |
| G | **Engine track.** Add the ranked engine list to the backlog behind the guardrails, starting with the kit-less sample game. | Design queues | Ongoing |

Recommended: A through E now, in that order, with A first because everything unattended depends on it. F and G follow once A lands.

**The question:** do you approve A to E as recommended? Type "all recs A-E", or name the ones you want, and say whether F and G should be queued behind them.

## Method and limits

Six read-only passes ran in parallel on `develop` at `11d320a`: the three pillars of the consistency-audit skill, plus workflow and planning, art and UI, and the engine. I ran the gate myself and viewed the founder, unit and skill-icon contact sheets. I spot-checked a sample of the other findings against the files; one claim from a pass (that the build lease branch was missing) was wrong and is not in this report, so treat individual line references as leads to confirm when each fix is built.

Not checked: the current look of the UI (no recent captures exist and I did not launch the game); audio by ear; `art_src/` images, which are LFS pointers in this checkout; `emberwake_gui` was swept by search, not read in full; the three legacy worlds' internals; the project-level skills and routines that live outside the repo; decision rows against each other line by line.

The cloud container needed two manual steps before the gate would run (a Linux build folder and the ALSA headers), because the repo's session-start hook did not run. An unattended session would have stopped there, so the environment setup script is worth fixing in step A.
