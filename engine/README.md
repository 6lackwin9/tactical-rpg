# Project Aegis Engine

This directory contains the source code for the **Aegis Engine**, an agent-native game engine written in Rust that runs in parallel with the Godot build and aims for feature parity with it.

**Current direction:** [docs/ENGINE_SPEC.md](docs/ENGINE_SPEC.md) (draft, 2026-10-18): headless deterministic sim, one command API shared by GUI and CLI, feel as data, parity with Godot (exact for rules, statistical for battles). All engine work stays under `engine/`; `.gdignore` keeps Godot from scanning it.

* **Original proposal (Markdown-as-language, superseded by the spec):** [ENGINE_PROPOSAL.md](ENGINE_PROPOSAL.md)
* **Active Branch:** `engine`
* **Toolchain:** Rust `stable-x86_64-pc-windows-gnullvm`, pinned in `rust-toolchain.toml`. The MinGW on PATH is llvm-mingw, and the plain `gnu` toolchain fails to link against it (`unable to find library -lgcc_eh`).
* **Build output:** `.cargo/config.toml` puts it in `C:/Users/tusha/AppData/Local/Temp/aegis-target`, outside OneDrive (the folder is several GB). Binaries: `.../aegis-target/release/aegis.exe`, `emberwake.exe`.
* **Game data:** the project's `games/emberwake/data/` folder (catalog, worlds, maps, quests, dialogue services, XP curve), read by the
  engine's own loader (`emberwake_content::source`) with the field tables in `kits/rpg/schema.json`; no Godot export is
  needed to play. `data=<dir>` points the GUI or CLI at another data folder, or at a Godot export folder.
* **Saves and settings:** per game under its id, `%APPDATA%/Aegis/emberwake/saves` and `%APPDATA%/Aegis/emberwake/settings.json` (outside the repo; the first start after SPLIT1-8 copies the old shared `%APPDATA%/Aegis/saves` and `settings.json` there and leaves them; `saves=<dir>` points the GUI elsewhere). The files use Godot's save format: Godot saves load in the engine unchanged. The engine owns the sound, display and text-size settings (`aegis_ui::screen::Settings`) and applies them when they change (PORT-SETTINGS): window mode (windowed at the chosen size, borderless or exclusive fullscreen), vsync, frame-rate cap and text size (Small 0.85 to Extra large 1.3) and the four volumes and mute (PORT-AUDIO); the game adds its own keys.
* **Drag and drop (UIKIT2):** a screen marks a widget draggable with a payload key (`Ui::draggable("stash.3")`) and widgets as drop targets (`Ui::drop_target("slot.weapon")`); `Screen::can_drop` says which targets take a payload (outlined while dragging, gold under the cursor) and `Screen::accept_drop` acts on a release over one. A press on a draggable widget clicks on release when the mouse did not move; a move of more than 3 canvas pixels picks it up (a ghost card follows the cursor, `Ui::dragging` lets the screen preview); Esc, right click or a release elsewhere cancels.
* **Windows (UIKIT3):** a screen can keep a `window::Desk` of windows inside an area (returned from `Screen::desk`): each window has a title bar to drag it, a corner grip to resize it, collapse and close buttons, and a body the screen fills in `Desk::draw` (drawing and widgets clipped to it). A press on a window raises it, and a window covers what lies under it (clicks, hover, drops). `Desk::toolbar` draws a toggle per window and a button per layout preset; the shell handles all of these itself (widget ids `win.<id>.<part>`, `desk.toggle.<id>`, `desk.layout.<name>`) and saves the desk in the settings under `desks.<screen id>` after each change (`Desk::restore` reads it back). Geometry is stored as fractions of the area. Agent link: `windows` lists them, `window {id, op: open | close | raise | collapse | move | size, by: [dx, dy]}` or `window {layout}` drives them.
* **Held keys (STAGE1-O):** `Cx::held` (`aegis_ui::Held`: shift, ctrl, alt) is the modifier keys held down right now; the shell keeps it current and clears it when the window loses focus, so a screen can show something only while a key is held (the battle outlines every unit while Alt is held).
* **Keyboard focus (PORT-FOCUS):** every screen's clickable widgets, choice fields and sliders take the focus with no
  code of their own (`aegis_ui::focus`, kept in the kit by the shell): Tab / Shift+Tab walk them in drawing order, the
  arrows move to the nearest one that way (a screen that walks or steers with the arrows says `Screen::arrow_focus`
  false: the region map, the journal), Enter or Space presses the focused one (a choice field opens its list and the
  arrows pick in it; Left / Right step a slider). A white ring shows the focus after a key moved it; a click hides it.
  `Screen::auto_focus` starts a keyboard menu focused (the main menu) on its first widget, or on
  `Screen::focus_first` when set. A question shows Yes then No, left to right, in both forms. A destructive question
  (`Cx::ask_no` / `ask_tag_no`, ASKNO1 and ASKNO2) opens with No focused, Enter = No and `y` = Yes, and through
  `Screen::pointer_to` asks the windowed shell to move the OS pointer onto No (only in a focused window, never for a
  screen an agent-link command opened, never in tests); `Cx::ask` / `ask_tag` keep Enter = Yes and open without a
  ring; the `ask` screen's state reports `default`. The agent link's `ui_tree` reports `focus` and
  `focus_visible`; `key` takes `backtab`; the link's `click` hides the ring like a mouse click. The focus starts over
  hidden on every new screen (a screen's own Enter / Space keep working); widgets under a popup or window take none;
  window title bars and toolbars take none yet, and a focused row is not scrolled into view.
* **Scroll areas and rich text (PORT-DOJO):** `Ui::scroll_begin(id, view, content_h)` / `scroll_end` clip content to a view and return the offset to draw it at; the kit keeps each area's offset (an overlay or a return keeps it; a new stack clears them all), the wheel closes an open choice list or scrolls the topmost area under the mouse (a window over it stops it; otherwise the screen's `wheel` gets the lines) and the agent link has `scroll {id, by | to}`. Clips nest (`push_clip` / `pop_clip`). `Painter::rich` draws markup (`[color=#rrggbb]..[/color]`, `[b]..[/b]`) wrapped to a width, `rich_height` measures it, `strip_markup` drops the tags. `Painter::arrow` draws a line (solid or dashed) with an arrowhead.
* **Timeline (TIMELINE1):** `aegis_ui::timeline::Timeline` draws a block timeline: a ruler of numbered blocks with a
  playhead, sections of labelled tracks (one row each, scrolling when they overflow), segments spanning blocks (filled
  or dashed, with a hover tip) and masked tracks (one hatched bar with a line of text). Blocks, track labels and
  segments are clickable (`<id>.block.<b>`, `<id>.track.<key>`, `<id>.seg.<track>.<i>` on the agent link).
  RULESET1-E: a segment can carry `parts` (`Part { start, end, style }`: `PartStyle::Windup` shaded, `Impact` bright
  with a light edge at its last block, `Recovery` hatched) or be `hatched` whole; a track can carry `links` (`Link {
  from, to, colour, tip }`, a bracket from one segment's end to the next one's start, hidden while masked) and `marks`
  (`Mark { key, block, glyph, colour, tip }`, a glyph chip inside its block's cell, shown on masked tracks too). Links
  and marks are info widgets with a hover card and no click; labels leave room for marks; all empty by default. The
  UI Gallery's Kit widgets entry has a Timeline page (tabs at its foot) showing every part. `timeline::draw_part(ui,
  style, bar, part_rect)` and `timeline::draw_mark(ui, id, &mark, rect)` draw one part or one mark on their own, so a
  screen's own block strip shares the widget's look (RULESET1-U slice 5). INPUT1-U: a segment can be a `ghost` (a
  planned, not chosen, span): its fill at `GHOST_ALPHA` (45% of its alpha), a dashed outline on all four sides and its
  label in `DIM`; `timeline::ghost_fill(fill)` and `timeline::draw_ghost(ui, bar)` give a screen's own spans the same
  look. Off by default (every existing timeline draws as before).
* **Vignette player (STORYUI1-E):** `aegis_ui::vignette` plays a reel of still cards (styles `image`, `text`, `title`,
  `line`) through fade in, typing, hold and fade out: a typewriter at the engine setting `text_speed` (`slow` 30,
  `normal` 60, `fast` 120 characters a second, `instant`; fast halves the fades, instant drops them), a cover fill of the
  frame colour for the fades, a press (click, Space, Enter) moving one phase, `hold_ms` above 0 moving on by itself, Skip
  (Esc, right click, the button; `ask_first` asks "Skip this scene?", Yes the default), a cue per card on `Bus::Sfx`, the
  reel's music wish, code-drawn gradient backdrops, letterboxed pictures and drifting ash (none under the shared
  `reduce_motion` setting). The model `Player` moves only through `advance(ms)`, `press()`, `skip()`; `vignette::draw`
  paints a card in the caller's `Palette`; `VignetteScreen<G>` measures frame time and runs `on_done` once. Agent link:
  `state` {reel, card, cards, phase, shown, total, done}, widgets `vignette.next`, `vignette.skip`. The UI Gallery's Kit
  group has a Vignette player entry on the engine's sample reel. PORTRAITFX1-D: a `line` card's portrait lives
  (`CardPortrait`: `portrait_fx` with its base life and its sidecar's `idle` overlays, no state holds or reactions;
  seeded by `Card.portrait_key`, else the drawn file's path; its clock moves with the frame time `VignetteScreen` feeds
  the reel; `VignetteScreen::portrait_rim` sets the rim key a sidecar does not name); `vignette::draw` draws it still,
  `draw_with` animates it; agent state `portraits` (the portrait on view, or none). PORTRAITFX1-X: `Card.expression`
  (read by `Card::from_json`; "" neutral) holds on the portrait as a `face` cue (`CARD_FACE_KEY`, `CARD_FACE_PRIORITY`
  100), so the agent state shows it; the caller's `portrait` chain already starts at that frame;
  `VignetteScreen::portrait_sidecars(fn)` (`CardPortrait::sidecars`) names the sidecar files to try for a drawn file
  (the game passes its kit's layout, so an expression frame without its own sidecar takes its base portrait's); without
  it the drawn file's own sidecar.
* **Stage pieces (STAGE1-E):** `Painter::ellipse_ring` (a 1 px ellipse outline, no fill), `Image::opaque_bounds(min_alpha)`
  (the box of solid pixels), `Art::body_bounds(key, palette, size, flip)` (the solid box of a body's fitted idle frame 0,
  mirrored with flip, cached; a unit's hit rect on a stage); DIR4-HB: `Art::body_bounds_facing(key, palette, size, facing)`
  (the box of `idle`'s front or back row for a `Facing` down or up, never mirrored; the side idle, mirrored when left,
  for a side facing or a body without that view row).
* **Effects (FX1-E):** `aegis_ui::fx`: the effect clock with hit-stop (`Clock`), six primitives drawn in code as pure
  functions of the clock and a seed (`Projectile`, `Beam`, `Trail`, `Sparks`, `Flash`, `Ring`; `Effect` with start, layer,
  seed), the live list `Fx` (at most 32), the camera shake (`Shake`, presets jolt / heavy / toll / tremor / quake in
  `ShakeTable`, `ShakeLimits` clamped to the engine ceiling 6 px / 3000 ms, `Camera::start` / `offset` with whole-pixel
  deterministic offsets; reduce motion starts none), `Painter::shift` (moves the world layer; clips and hit rects stay),
  `Image::silhouette` / `Art::body_silhouette` (the hit flash), `Settings::reduce_motion`. STATUSFX1-E adds `Orbit` (dots circling a point on an ellipse, the back half at alpha 110, a star or a square; a star's `size` is its arm, 1 the 3 px plus and 2 a 5 px plus since STATUSFX1-L), a closing `Ring` (`r0 > r1`), `Looped` (an effect repeated every `period`, reseeded per cycle, drawn by the caller every frame outside the live list; `draw_still` for reduce motion) and `Painter::image_alpha`; the UI Gallery's Effects page gains `kit.fx.orbit` and a loop toggle `kit.fx.loop`. The UI Gallery's Kit widgets entry
  has an Effects page (a button per primitive and preset, speed, hit-stop, reduce motion).
* **Portrait effects (PORTRAITFX1-E):** `aegis_ui::portrait_fx` draws a portrait through a `PortraitFx`: base life (breath, rim flicker, blink), overlays (glow, veins, eye glow, ash, smoke), states (desaturate, tint, fade, still) and reactions (flash, a jolt with `hit`/`crit`, at most 4 px), placed by the face sidecar `<portrait>.face.json` (`aegis_core::assets::parse_face`, read by `Art::face`; `Art::first_rel` names the file a chain drew). Size tiers (full from 96 device px, reduced 48-95, minimal below), `Settings::reduce_motion` (motion stops, information stays), composed on the CPU at the drawn size (at most 256 px), cached, at most 30 steps a second and at most 8 time-varying portraits a frame (`Budget`); every cue level and time-varying effect reads the clock in 30 steps a second, and periodic motion takes t mod period first (`cycle`); a sidecar `idle` list holds overlays only; `state(now)` for the agent link; the Kit widgets Portraits page shows a code-drawn sample face with every toggle. The sidecars are written and checked by `tools/portraits/face.py`; `aegis_core/tests/face_fixtures.rs` keeps its rules matching the engine's (run `python tools/portraits/test_face.py` after changing the tool: the gate runs only the Rust half).
* **Sound and music (PORT-AUDIO):** `aegis_ui::audio` plays the `audio/` files found over the asset layers (game, kit, engine; ASSETORG2) (WAV and Ogg Vorbis, through rodio) on four levels (master times music, effects or interface), 8 effect voices and 2 interface voices (the oldest restarts when all are busy), and one looping music track that crossfades (1 s; 2 s from silence; 6 dB under the effects). Each screen names its track (`Screen::music`); the shell plays the topmost screen's choice, so overlays keep the music below. Emberwake picks tracks with the MUSIC1 chain (`emberwake_gui/src/music.rs`: menu, region by location kind or shop, travel, combat or boss, each tried per region, world and lineage before the plain context) and battle sounds from the resolver log; the settings popup plays the draft levels and a sample per volume slider. Without a sound device everything stays silent.

## Workspace

| Crate | Role |
| :--- | :--- |
| `crates/aegis_core/` | Engine core, game-agnostic (SPLIT1-2): Godot RNG, sort and `hash()`, Variant helpers (`json`), lenient field readers (`value`), Godot number formatting (`fmt::fixed`: `%.Nf` with ties away from zero); DIR4-E direction views: clips.json rows with `of` + `view` (`down`/`up`, checks V1-V4), `View`, `Facing` (`toward(dx, dy)`: up/down when the vertical distance is larger, a diagonal tie keeps the side view and the last left/right; `mirrors`), `Clips::resolve_view` / `view_row` / `frame_at_view` (view row, side row, idle in that view, side idle, first row), `ClipFrame.side`, `ClipPlayer::frame_facing`; DERIVE1-E formula evaluator (`formula`): `Formula::parse` (+ - * /, unary minus, parentheses, min/max/clamp/floor/ceil/round; errors with a 1-based column; limits 256 chars, 16 nesting, 64 tokens), `names()` (first appearance order), `eval(&mut dyn FnMut(&str) -> f64)` (f64, `x / 0` = 0, never NaN or infinity; callers round), `is_name` / `is_function`; `assets::palette_problems(&Layers)` parses every palette file (`units/<class>/<variant>/palettes/*.json`) in every layer, referenced or not, with the loader's rules (`parse_palette`), one sorted `<path>: <reason>` line per problem; the game test `asset_checks::every_shipped_palette_file_parses` runs it on the shipped layers, so the gate fails on any broken palette (ASSETCHK1) |; FX1-L1: a clips.json row may carry `impact` (a frame of the clip, checked 0..frames-1), `Clip::ms_before`, `Clip::impact_ms`; WINDUP1-E: a row may carry `loop_from` (a looping clip's intro frames play once, then `loop_from..frames` loop; 1..frames-1, loop only) and `strike` (0..frames-1, at most `impact`), `Clip::intro_ms` / `frame_at_still`, `Clips::ms_before(id, frame)`, `ClipPlayer::play_from(clip, now, frame)` (a shot from a frame, shorter by the skipped ms) and `set_base_from(clip, now, from, still)` (the base from a frame; `still` holds frame `loop_from` after the intro; `set_base` = from 0, not still)
| `crates/aegis_content/` | Engine content layer, game-agnostic (SPLIT1-3): data file format, schema normalize and storage, content store and world graph, upsert / remove over the game's hooks |
| `crates/aegis_save/` | Engine save framework, game-agnostic (SPLIT1-4): slot files, envelope, atomic write, migration runner, slot headers; slot locks (PLAYUI1-S): `is_locked` / `set_locked(fmt, dir, slot, on)` in `<save dir>/locks.json` `{"locked": [..]}` (sorted, manual slots only, an entry counts only while its save exists); `write` and `delete` refuse a locked slot (`Slot <n> is locked.`), the autosave is never locked. A disabled kit icon button swallows its click, so a row under it never fires (`ui::hit`). Keyed saves (CAMPAIGN1-E): `write_keyed` / `read_keyed` / `exists_keyed` / `peek_keyed(fmt, dir, key)` in `<save dir>/key_<key>.json`, the same envelope and safe write as a slot; a key is 1-64 characters a-z 0-9 `_` `-` (`check_key`); outside the slot range, so slot saves, locks, `delete` and `newest_slot` never touch or list them; a rewrite with the same key replaces the save (the act-start save `act<n>_<run>`). |
| `../games/emberwake/crates/emberwake_data/` | Typed content catalog (classes, units, skills, items, passives, encounters, terrain) |
| `../games/emberwake/crates/emberwake_rules/` | Pure rules: stats, gear, unit building, damage, statuses |
| `../games/emberwake/crates/emberwake_battle/` | Battle sim: grid, paths, commands, resolver, planner, AI, session, action log, battle metrics and report, balance-sim fight |
| `../games/emberwake/crates/emberwake_content/` | The game's data loader and writer on `aegis_content` (`source` reads `games/emberwake/data/` like Godot's catalogs, `store` writes it like Godot's saves) |
| `../games/emberwake/crates/emberwake_world/` | The game loop around battles: game state, new game, saves (Godot format, every migration step, on `aegis_save`), exploration, quests, dialogue, shops, travel, rest, battle bridge |
| `crates/aegis_host/` | Engine command API protocol, game-agnostic (SPLIT1-6): JSON request in, JSON reply out, history and step counter over the game's command set |
| `../games/emberwake/crates/emberwake_sim/` | The balance simulator (PORT-SIM): sandbox units, the nine modes (duel, tier, levels, group, stats, party, role, sweep, campaign), the engine-only `feel` (RULESET1-G) and the engine-only, fightless `deadlines` (CAMPAIGN1-SIM, in the kit's `rpgkit_sim`), scenarios, roles, targets and KPI tables; `aegis sim <mode>` prints its lines |
| `../games/emberwake/crates/emberwake_host/` | The game's command set (state, skills, plan, preset, stay_pick, confirm, step, resolve, advance, autoplay) and CLI commands; holds the `aegis` binary (SPLIT1-6) |
| `crates/aegis_cli/` | Engine command line shell (SPLIT1-6; the binary is built from `emberwake_host`; `content manifest --kit <id>` writes a kit's kit.json, ASSETORG2): the game's `sim <mode>`, `battle <encounter>`, and `serve` (the command API over stdin / stdout), `link` (the GUI's agent link), `content` (edit `games/emberwake/data/`: check, upsert, remove, apply; `content world <id> [<json | @file>] [full]` shows, creates or updates a world manifest `data/worlds/<id>/world.json` (WORLDW1): fields merge over the stored manifest or the schema defaults, values are checked for type and enum options, and the file is kept only when the data folder gains no blocking or manifest problem, else rolled back; without `full`, fields missing from the file stay out while at their default (a resave is byte-identical), `full` writes every field; seed step `{"op": "world", "id", "fields", "full"?}`; a new world is listed in game.json `worlds` first with `content manifest`) |
| `crates/aegis_ui/` | Engine UI, game-agnostic (SPLIT1-5): canvas and painter, widget kit, art loader over the asset layers (ASSETORG2: the game's `assets/`, its kit's `kits/<kit>/assets/`, the engine's `engine/assets/`, first hit wins; files by relative path, bodies as clip sheets with `clips.json` and palettes, the token = the idle clip's first frame, tiles by the variant files present), audio over the same layers, live tuning files, agent link, screen stack with the game's context, window shell and event loop; clip playback (SPRITE1-L): `aegis_core::assets::ClipPlayer` (a base loop plus one shot, a missing clip falls back to `idle`, then the first frame) drawn by `Art::played` (`flip`: mirrored left to right, cached apart; `body_clip_flipped`; `body_outline` takes the same `flip`, SPRITE2), cached silhouette outlines (`Art::body_outline`, `Image::outline`), body checks frame by frame (`Art::body_problems`), `Painter::ellipse`; PROPS1-E: `aegis_ui::depth::DepthList` (sort by feet, then group, then x, then push order), `Image::mirrored` / `Art::image_mirrored`, `Art::size_of` (a PNG's own size), `Art::varied` / `Art::pick` (the first folder with `v0.png`, an explicit variant or the position pick; `tile_in` wraps it); DIR4-E: `Art::played_facing` (a four-way `Facing`; only side rows mirrored, for left) |
| `../games/emberwake/crates/emberwake_gui/` | `emberwake`: every game screen (menus, region map, dialogue, shop, world map, game menu, character, journal, results, battle) and the dev tools (Scene Runner, Content Forge, World Forge, Gear Dojo, UI Gallery) on `aegis_ui`; agent link on port 47800 |
| `crates/aegis_app/` | Retired 2026-10-19 (transition step 2): the old egui demo, superseded by `emberwake_gui`; in git history |
| `crates/aegis_core/` (old) | Retired 2026-10-19 with `aegis_app` (its only user); in git history. The name is reused by the engine core above (SPLIT1-2) |

## Parity with Godot

The Godot game is the reference: the frozen copy in `legacy/` (main at 7fa10da, see `legacy/LEGACY.md`; branch
`legacy` to resume Godot work). `parity/` holds the tools that run it and check the engine against it (new files only;
no Godot file is edited):

```bash
games/emberwake/parity/export.sh                      # catalog, worlds, game data, schema, unit snapshots, damage cases, RNG, save fixtures (~5 s)
games/emberwake/parity/trace.sh mode=duel level=5    # record Godot sim fights step by step (duel | group | teams | encounters)
games/emberwake/parity/trace.sh mode=duel level=5 pouch=con_heal_potion,con_fire_bomb   # the same with battle items (party AI uses them)
games/emberwake/parity/trace.sh mode=world preset=withdraw   # every world encounter as WorldEncounter plays it (objectives, phases, morale)
games/emberwake/parity/journey.sh seeds=1,2,3,4 steps=700   # record seeded random game-loop journeys (~4 min)
games/emberwake/parity/sim.sh                         # record the Godot balance sim's tables for 20 command lines (~25 min)
games/emberwake/parity/checks.sh                      # record the Godot content checks: broken entries, world lints, reports (~10 s)
cargo test -p emberwake_rules -p emberwake_battle -p emberwake_host -p emberwake_world -p emberwake_sim -p emberwake_content
    # exact parity: the data loader, units, damage, RNG, battle, item and world battle traces (with their action logs,
    # metrics and reports), journeys (result, state and screen views per action), saves, the sim's printed tables,
    # the content checks (errors and warnings, message for message)
cargo run --release -p emberwake_host -- sim duel 1 20   # the balance-sim duel table (sim help: every mode and key)
tools/fc.sh sim feel [enc=a,b] [seeds=N] [ruleset=..] [rules=-id,+id]
    # RULESET1-G, engine-only (not in the sim help): catalog encounters with their party (default the game's five tl_
    # encounters, 10 seeds, timeline): breaks, windups, flanks, pierce, AP, intents, Rest, carried moves, kiting,
    # planner reasons per encounter and in total; last line FEEL_JSON {...} (machine-readable, for RULESET1-B)
tools/fc.sh sim deadlines [world=<id>] [losses=0..9]
    # CAMPAIGN1-SIM, engine-only, no fights (not in the sim help): per campaign world with checkpoints, the best and
    # worst day each checkpoint's quest can be won and the slack (CAMPAIGN1 C5); no checkpoints: one line says so
```

Status and the file-by-file map: [docs/PORT_NOTES.md](docs/PORT_NOTES.md).

## Driving a battle from the CLI (agents)

`aegis serve` reads one JSON request per line and answers one JSON line each (full list in
`../kits/rpg/crates/rpgkit_host/src/lib.rs`, KIT1-N):

```bash
printf '%s\n' '{"cmd":"new","encounter":"goblin_skirmish"}' \
  '{"cmd":"plan","unit":"Warrior","steps":[{"mode":"engage","skill":"strike","target":"Goblin Brawler"}]}' \
  '{"cmd":"preset","unit":"Ranger","preset":"focus_weakest"}' '{"cmd":"resolve"}' | aegis serve
```

Every mutating request is recorded; `{"cmd":"history"}` returns them (and the checkpoints: the round outcome after
every concluded round), and feeding them to a fresh `serve` replays the battle exactly.

`new` also takes `ruleset` (`classic` or `timeline`; default classic for a catalog encounter, the world's for a world
encounter) and `rules` (`"-id,+id"`, extensions only; RULESET1-R); the CLI takes the same as
`aegis battle <encounter> ruleset=<id> rules=<-id,+id>` and the sim as `ruleset=` / `rules=`.
`new` also takes `plan_reveal`; under the rule `plan_badges` the `intents` reply gains `round_intents` (both sides: intent, target, Maneuver kind, impact badges, tell, plan) and `intel {unit}` gives one unit's record (RULESET1-V).

## Playtest recordings and replay (PORT-PLAYTEST)

While recording is on (the setting `record_playtests`, on in debug builds), the GUI keeps one recording per run in
`%APPDATA%/Aegis/<game>/playtests/<id>.json`: every arena battle's command-host session (its requests from `new`
and the checkpoints), saved when the battle ends, restarts or the screen closes, with the content fingerprint of the
data folder (`aegis_host::playtest`, the hub `aegis_ui::playtest`), and the UI trail: clicks (the widget, handled or
not), widget presses, keys, hovers of 700 ms or more, screen changes, overlays as modals, plan edits, confirms, the
battle's end, errors and panics, and missing art (`aegis_host::trail` reads it). Party battles of the world map
start outside `new`, so their replay waits for PLAYTEST2.

F8 (recording on) takes a note: the screen as it was goes to `<id>/note_<n>.jpg`, and a box asks for the text (types
at once) and tags (mechanic, ui, ux, bug); Ctrl+Enter saves, Esc cancels. The note stamps the screen, mouse, control
and the battle's round, block and phase. The kit's text field (`Ui::text_field`, single or multi-line) edits through
the shell (`Kit.edit`, `Screen::text_changed`) with a full editor (PLAYUI1-T, `aegis_ui::edit`, re-exported from `ui`):
it wraps at spaces and scrolls (`<id>.scroll`); Up/Down by visual line (sticky column), Home/End, Ctrl+Home/End,
Page Up/Down, Ctrl+Left/Right, Ctrl+Backspace/Delete, Shift selects, mouse drag, Shift+click, double click selects a
word, Ctrl+A/C/X/V, Ctrl+Z/Y; undo groups a typing run (a 1 s pause, a move or a delete ends it; 100 steps); the
clipboard goes through `arboard` with an in-app fallback. The F8 note box is a full-height text area. The kit also has
an icon button (`Ui::icon_button`: an image from the asset layers or a drawn `Glyph`, hover card, focus, toggled and
disabled states) and a collapsible section header (`Ui::section`) (PLAYUI1-K).

The Scene Runner's **Playtests** (`aegis_ui::recordings`, a kit screen the game opens with its own launcher) toggles
recording and lists the user folder's recordings and the game's repo folders (`playtests/inbox`, `playtests/repro`,
badge `repo`) with a stale badge: per battle Play back (watch it to its end, not recorded) or Take over at a round
(replayed, then played on as a new session whose `end.parent` names the recording, battle and request), the notes
(click a note's row, `recordings.note.<i>`, to read it whole in `recordings.note_view` with Copy), Open folder and Copy to repo (the recording and its images into `playtests/inbox/<id>/`).

```bash
aegis replay <recording.json>                       # every battle: each checkpoint match / diverge (path, values) / unverified
aegis replay <file> battle=0 until=12 json          # stop after 12 requests (what a take-over replays), JSON report
aegis replay <file> variant=patch.json write=out/   # {"at", "remove", "requests", "then": "stop"|"recorded"}: DIFF lines, saved run
aegis replay <file> seeds=1..50 fallback='{"cmd":"autoplay","preset":"balanced"}'   # results under other seeds
aegis replay <file> trail-report                    # the UI trail's flags: dead or repeated clicks, long hovers, errors, panels
```

Exit 0 when everything matches (changed data files only warn: "content stale"), 1 on a divergence or a refused
request, 2 when the recording does not load. `tools/fc.sh replay ...` is the same.

## The game (emberwake)

```bash
cargo run --release -p emberwake_gui                       # the main menu; screen=runner (dev tools) or screen=arena
cargo run --release -p emberwake_gui -- screen=arena encounter=goblin_ambush seed=3
```

The main menu leads to New game (world cards), Continue / Load, Settings and Tools (the Scene Runner: Combat Arena on
any catalog or world encounter (World / Classic / Timeline rules, single extensions switched, "Reveal enemy plans", a
ruleset chip: RULESET1-G), Content Forge, World Forge, Gear & Skill Dojo, Exploration (a world from any location or its world map,
nothing saved, a fight skipped as a win at no cost or fought), and the UI Gallery: the kit's widgets, every screen and the battle's panels one at a time). In play: arrows / W A S D walk, E interacts with the marker you face, a click walks there
(or uses a marker), Esc opens the game menu (party, journal, save, load, settings, quit). Both maps have a HUD bar
(PLAYUI1-H): the place name top left; gold, supplies and icon buttons top right (P party, J journal, I stash = the
Character popup's Gear tab, ? help, Menu), off while a conversation, shop, turn-in or lore page is open. Talking opens the dialogue box at the bottom centre with the camera on both speakers (1-9 choose); shops, turn-ins and lore open as popups; a world-map exit opens the world map (click a node, Travel,
Make camp, Enter). Fights open the battle screen with the saved party; Continue settles the outcome and shows the
results. The screen list and what is left: [games/emberwake/docs/SCREENS.md](../games/emberwake/docs/SCREENS.md).

In battle: hover anything (units, cards, plan blocks, menu options) for a detail card. Click a hero to select it; click
a foe, an ally, the hero itself or an empty tile for its options (each shows AP and the expected damage or heal); right
click closes a menu or undoes the last step. Strip: the Party and Hero dropdowns draft a tactic (with its
description), Auto plays the battle out, Confirm resolves the round. The log (Expand) wraps and scrolls with the wheel
or PageUp / PageDown. Keys (while no control shows the focus ring): Enter confirm, Space step / next round, A autoplay, Z undo, L log, R restart (arena),
I enemy tells, F12 screenshot. Timeline battles (INPUT1-U): click a target (an enemy, an ally, an empty tile; the hero
panel's Self button or S for the hero itself), then pick skills from its menu (keys 1-9 are the selected hero's skill
hotkeys, with or without the menu open); the planner fills the rest of the round as ghost blocks (Settings, Gameplay:
"Auto-fill the rest"); Z or a right click undoes the last pick, Clear removes them all; a right click or a click outside
closes the menu. Selection and inspect (INPUT1-U slice 3): a right click on any unit (token or card) inspects it, pinning
the threat intel panel ("Inspecting `<name>` (Esc clears)") until another inspect, Esc or the next round, with only what
the party knows (an enemy's unrevealed plan never shows); a click on the selected hero (token or card) selects nothing;
with no hero selected, or outside planning, a click on an enemy inspects it and one on an ally selects it; Esc closes
the menu, then the log, then clears the inspect, then selects nothing, then (Arena) leaves.

Feel values (timings, floating numbers, hit flash, colours) live in `games/emberwake/tuning.json` and reload while the game
runs. The GUI serves an agent link on `127.0.0.1:47800`: `ui_tree` (the screen stack, the music wanted, the top screen's state and every
widget with its id, text, rect, state, hover card and drag or drop key), `click {id}` (a select opens its list and replies with the option ids, `<id>.opt.<n>`), `drag {id, to}` (press, move, release to `[x, y]` or onto a widget id: slider drags and drag and drop), `hover {id}`, `type {text}` (into the text field being edited), `note` (F8), `key {key}` (up / down / left /
right / enter / space / esc / a letter; while a field edits also home, end, backspace, delete, `shift+...` and
`ctrl+a/c/x/v/z/y`, and enter, esc and tab end the edit; `ui_tree` then reports `edit` {id, caret, selection, selected,
undo}), `back`, `screenshot {path, max_w}`, `text_audit {scale, shot}` (TEXTFIT1: one
frame of the top screen at a text size, the mouse off the canvas, reporting the text `text_fit` or a button shortened
(cut), text past the canvas (edge), texts drawn over each other (overlap) and text running out of its control (spill);
text in a scroll area or under a later panel is skipped, and so is transient text drawn inside `Painter::transient`
(TEXTFIT2: floats that rise and fade, such as damage numbers; the painter flag `Fonts::transient` marks the record
`TextRec::transient`); `shot` saves that frame), `game` (the full game state), and the
battle command API while a battle is open. An agent sees and does exactly what the player sees and does:

```bash
aegis link '{"cmd":"ui_tree"}' '{"cmd":"click","id":"menu.new_game"}' '{"cmd":"click","id":"new_game.start"}' '{"cmd":"key","key":"right"}'
```
