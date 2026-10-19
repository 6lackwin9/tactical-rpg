# S14-RULES: asset pipeline, animation, audio and asset-load tests (baseline)

Status: proposed (design-brainstorm 2026-10-17; baseline rules so S14 can be specced and built later; nothing approved)
Depends on: none for the art registry; S12-RULES status and terrain ids for their icons; UI_MENUS_RULES settings for volumes and animation speed
Lane: design (this file); later logic and asset tasks listed in section 4
Touches (when built): `scripts/art/*` (UnitArt, ItemArt, TileArt, SvgRaster), new `scripts/art/asset_catalog.gd` (Q-A1), new `data/assets/manifest.json`, `assets/` (placeholder folder, STRUCTURE.md), `scripts/content/validators/class_unit_validator.gd` (art key check), `scripts/content/asset_scan.gd`, new presenter and audio scripts under `scripts/ui/combat/` and `scripts/audio/` (new path: planner approval), new suites

User direction: core systems including the HANDLING of art assets are the focus; placeholder art stays until the base game is done (DECISIONS workflow "M2 slice order"; "Cleanup pass 1 answers" (2)); units without their own art may reuse any placeholder (balance.md "Small answers", CLN-15).

## 1. Scope and what it is NOT

In scope:

- How content refers to art and sound (keys), how a key resolves to a resource (file override, generated placeholder, fallback), validation and safe delete of keys.
- Animation as presentation of resolved combat events; an instant mode for tests and sims.
- Audio buses, a sound bank by key, silent fallback.
- The asset-load and animation lifecycle test suite (README "Test Suite 2").

NOT in scope: final art, art style, music or SFX production, sourcing or licences of files, a UI theme pass, sprite or skeletal animation content (Q-A2 reserves the data only), VFX shaders, voice. Gameplay never depends on animation timing or the scene tree (STRUCTURE.md conventions); nothing here changes rules, balance or the sim.

## 2. Baseline rules

### 2.1 What exists today (generated placeholders)

| Asset | Code | Key source | Native size (SVG viewBox, rasterised x2) |
| --- | --- | --- | --- |
| Unit token | `UnitArt.token(key, team)` | class `art` / unit `art` (empty = class's), validated against `UnitArt.CLASS_KEYS` (12 keys incl. goblin archer, berserker, shaman, skirmisher, dummy) | 64 |
| Unit portrait | `UnitArt.portrait(key, team)` | same key | 120 |
| Item icon | `ItemArt.icon(item)` | `icon_id` if in `ItemArt.UNIQUE_ICONS` (2 ids), else derived from gear type (`icon_key`) | 48, frame tinted by rarity |
| Terrain tile | `TileArt.tile_at(type, coord)` | `Terrain.Type`, 4 variants, overlays by kind | 54 |
| Cache | `SvgRaster.texture(key, svg, scale)` | cache key `key@scale` | - |

Team identity (blue, red, yellow; spiked or dashed rim) is drawn by the template, not stored per unit. Skills, statuses, world characters, encounter backgrounds, sounds and animations have no keys yet.

### 2.2 Key rules (all kinds)

1. Content references art and sound by KEY, never by file path. A key is `snake_case`, unique inside its namespace.
2. Namespaces and variants:

| Namespace | Referenced by | Variants | Default key rule |
| --- | --- | --- | --- |
| `unit` | class `art`, unit `art`, world character `art` | `token`, `portrait` | unit's own, else class's, else `unit` (today's `ContentFactory.art_of`) |
| `item` | item `icon_id` | `icon` | `icon_id`, else derived gear-type key (today's `icon_key`) |
| `tile` | `Terrain.Type` | `base` x4, `overlay` | type name in snake_case |
| `skill` | skill family | `icon` | family id; optional override field `icon_id` on the family (same pattern as items) |
| `status` | `StatusRegistry` row | `icon` | status id |
| `fx` | animation presets (2.4) | `preset` | action kind (`move`, `attack_melee`, `attack_ranged`, `cast`, `hit`, `defend`, `heal`, `status`, `defeat`) |
| `sfx` | sound events (2.5) | `stream` | event name |
| `music` | zones and menus | `stream` | zone id, `menu` |
| `ui` | menu screens | `icon`, `background` | screen id |

3. Resolution order for any key and variant: (a) a manifest file override (2.3), (b) the generated placeholder template if the namespace has one, (c) the namespace fallback (`unit`, `weapon`, a neutral tile, a blank status or skill disc, silence for sound). Resolution never returns null to the UI; a missing override logs one warning per key per session.
4. Team tint is an overlay for file art too: a unit file is drawn untinted and the token rim (team colour plus spike or dash pattern) is composed by code, so one file serves all three sides and colour-blind readability stays.
5. Reuse is explicit: a manifest row may alias another key (`alias_of`), the data form of the CLN-15 answer (goblin_brawler reuses berserker).
6. Validation: a content field holding a key is valid when the key exists in the namespace (generated template OR manifest row). `ClassUnitValidator` stops reading `UnitArt.CLASS_KEYS` directly and asks the registry. Forge pickers (`options_from: art`) list registry keys.
7. Safe delete: `AssetScan` counts key references; deleting a manifest row that shipped content references is refused (same rule as content ids).

### 2.3 Manifest (data model sketch; where it lives is Q-A1)

One row per key and variant that has a FILE (generated-only keys need no row):

| Field | Type | Default | Validation |
| --- | --- | --- | --- |
| `key` | string | required | snake_case, unique in namespace |
| `namespace` | enum (2.2 list) | required | - |
| `variant` | enum per namespace | first variant | must belong to the namespace |
| `path` | string | "" | `res://assets/...`; file must exist and load; empty only with `alias_of` |
| `alias_of` | string | "" | an existing key of the same namespace; no alias chains (one hop) |
| `size` | `[w, h]` | namespace default | loaded texture must match; warning (not error) when it does not |
| `frames` | int | 1 | reserved for sprite strips (Q-A2 option B); must be 1 until that slice |
| `origin` | `system`, `user` | `user` | same origin rules as content (SKILL1) |

File layout: `assets/art/<namespace>/<key>_<variant>.<png or svg>` (Godot; the engine since ASSETORG1: `assets/art/<namespace>/<key>/<variant>.png`), `assets/audio/sfx/<key>.ogg`, `assets/audio/music/<key>.ogg`. Godot import settings: art imported lossless with filtering per namespace (tiles nearest, portraits linear), audio OGG Vorbis, music loops on.

Default sizes for file art (pixels; the placeholder x2 raster sizes, so swapping keeps layouts): token 128x128, portrait 240x240, item icon 96x96, tile 108x108, skill and status icon 64x64.

### 2.4 Animation (presentation only)

- Source of truth is the resolved round: the resolver's `BattleCommand` results, reactions (riposte) and log lines, in block order. A `CombatPresenter` (scene-side) turns each event into a preset from the `fx` namespace; the logic never waits for it and undo/replay re-run presentation from the same events (the existing playback controls: step, play, reset).
- Presets are tween recipes on static tokens (slide along the path, bump toward target, hit flash and shake, floating numbers, status icon pop, fade on defeat), each with a duration in a data table; a skill family may name an `fx_id` override.
- Speed setting: `anim_speed` multiplier (UI settings) and `instant` mode. Tests, the sim and headless runs always use instant mode: every preset completes synchronously and frees its nodes in the same frame.
- Lifecycle rules: every tween is owned by the presenter, killed on round reset, scene change or undo; temporary nodes (numbers, icons) are freed by the presenter; a queue of more than one round of events collapses to instant to avoid lag.

### 2.5 Audio

- Buses: `Master`, `Music`, `SFX`, `UI`; volumes from settings (`user://settings.cfg`, not the save file; owned by UI_MENUS_RULES).
- `SoundBank.play(event_key)` resolves `sfx/<key>` through the registry; a missing stream is silent with one warning per key per session. Events come from the same presenter event stream (hit, crit, block, heal, status applied, defeat, move step) plus UI events (click, confirm, back, error).
- Music: one stream per zone or menu key, cross-fade on change, no stacking. Headless and sim runs never instantiate audio players.

## 3. Open design questions (only choices that block a spec)

Q-A1 Where keys and file overrides are declared.

- A: code constants (today: `CLASS_KEYS`, `UNIQUE_ICONS`). No pipeline, but every new art file is a code change and validators read code lists.
- B: a data manifest `data/assets/manifest.json` behind its own small `AssetCatalog` (upsert, validation, origin, safe delete), the precedent being the separate QuestCatalog (DECISIONS world-campaign "M4 quest answers"). Asset lane adds art without code; no content-hub conflict with ITEM1/CLASS1.
- C: folder convention only (any `assets/art/unit/<key>_token.png` is a key). Zero data, but typos become new keys, aliases and origin are impossible, AssetScan has nothing to scan.
- Recommendation: B, with the C file layout as the required path pattern.

Q-A2 Animation depth for the baseline.

- A: tween presets on static tokens (2.4). Works with placeholder SVGs, fully testable headless in instant mode.
- B: A plus sprite strips per unit key (idle, attack, hit, defeat) through `frames`. Needs real art to be worth it.
- C: per-unit AnimationPlayer scenes. Heaviest; couples content to scenes.
- Recommendation: A; B reserved through the `frames` field.

Q-A3 First audio slice.

- A: buses, SoundBank, combat and UI SFX keys with silent fallback; no music.
- B: A plus zone and menu music with cross-fade.
- Recommendation: A (no audio files exist yet; the wiring and tests are the core part); B with the first real audio files.

Q-A4 Skill and status icons: generated placeholders or fallback only.

- A: generated placeholder discs (status: registry kind colour plus first letter; skill: kind glyph plus tier pips) so the planner and inspector show something distinct now.
- B: plain fallback disc until real icons exist.
- Recommendation: A; it is small SVG work in `scripts/art/` and makes the skill-tier picker (ROADMAP open engine item) readable.

## 4. Milestone shape (coarse; each needs user approval)

| Task | Lane | Content | Depends on |
| --- | --- | --- | --- |
| S14-L1 | logic, schema first | `AssetCatalog` + manifest schema (Q-A1), registry resolution (override, template, fallback), team overlay for file art, validators and Forge pickers read the registry, AssetScan, skill and status placeholder icons (Q-A4), suite `asset_pipeline_test_suite.gd` | none; after ITEM1-L if it touches `ItemArt` callers |
| S14-A1 | asset | manifest rows: aliases for existing reuse (CLN-15 units), one sample file override per namespace to exercise the path (placeholder PNG exported from the SVG) | S14-L1 commit 1 |
| S14-L2 | logic | `CombatPresenter`, `fx` preset table, instant mode, lifecycle rules, suite `animation_lifecycle_test_suite.gd` | S14-L1; UI1-L settings for `anim_speed` (can default until then) |
| S14-L3 | logic | buses, `SoundBank`, settings volumes, event wiring | S14-L1; UI1-L settings |
| S14-A2 | asset | first SFX files and manifest rows (source and licence chosen by the user) | S14-L3 |

Rule-derived acceptance checks for the suites:

1. Every key referenced by shipped content (classes, units, world characters, items, skill families, statuses, terrain types) resolves to a non-null texture whose size equals the namespace default or its manifest `size`.
2. Every manifest row's `path` exists and loads; an alias row resolves to its target's texture; an alias chain is a validation error.
3. Removing a manifest file at runtime falls back to the template (or namespace fallback) with exactly one warning per key.
4. Two lookups of one key and scale return the same cached instance; `SvgRaster.clear_cache` forces a rebuild.
5. A unit file override renders identically for all three teams except the composed rim.
6. In instant mode a full replayed round leaves zero live tweens and zero presenter-owned temporary nodes; a reset mid-playback kills every tween (no callback fires after reset).
7. A missing `sfx` key plays nothing and does not error; headless runs create no audio players.
8. Deleting a referenced manifest row is refused by safe delete.

## 5. HISTORY rows and decisions that touch S14

| Row | Effect |
| --- | --- |
| HISTORY 8, hard-coded content replaced by the JSON catalog | same reason applies to art keys in code constants (Q-A1 B) |
| HISTORY 23, off-hand shortsword category deprecated | `ItemArt.icon_key` still reads `off_hand_category == "shortsword"`, a leftover of the deprecated concept; S14-L1 (or the housekeeper) should derive the icon from the family hand instead |
| DECISIONS workflow "M2 slice order" | art stays placeholder until the base game is done; this baseline builds handling, not art |
| DECISIONS balance "Small answers", CLN-15 | reuse of placeholder art becomes manifest `alias_of` |
| DECISIONS world-campaign "M4 quest answers" | separate-catalog precedent for `AssetCatalog` |

No HISTORY row deprecates an audio or animation idea.
